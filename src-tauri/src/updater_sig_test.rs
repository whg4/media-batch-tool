//! Verifies the Tauri updater signature chain with the exact logic the
//! tauri-plugin-updater uses: base64(pubkey) -> PublicKey::decode,
//! base64(signature) -> Signature::decode, then verify.
//!
//! Run with the real release artifacts:
//!   MBT_SIG_DMG=target/release/bundle/dmg/MediaBatchTool_0.1.0_aarch64.dmg \
//!   MBT_SIG_FILE=target/release/bundle/dmg/MediaBatchTool_0.1.0_aarch64.dmg.sig \
//!   cargo test -- updater_signature
#![cfg(test)]
use base64::Engine;

#[test]
fn updater_signature_verifies() {
    let dmg = match std::env::var("MBT_SIG_DMG") {
        Ok(p) => p,
        Err(_) => return, // skipped without artifacts
    };
    let sig_file = match std::env::var("MBT_SIG_FILE") {
        Ok(p) => p,
        Err(_) => return,
    };

    // 1. read pubkey from tauri.conf.json (same source the app uses)
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let conf: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(format!("{manifest_dir}/tauri.conf.json")).unwrap(),
    )
    .unwrap();
    let pubkey_b64 = conf["plugins"]["updater"]["pubkey"].as_str().unwrap().to_string();

    // 2. the .sig file IS the base64 of the minisign SignatureBox text
    //    (the same value that goes into latest.json "signature")
    let signature_b64 = std::fs::read_to_string(&sig_file).unwrap().trim().to_string();

    // 3. exact same verification as tauri-plugin-updater::updater::verify_signature
    let pub_key_decoded = base64::engine::general_purpose::STANDARD
        .decode(pubkey_b64)
        .expect("pubkey must be base64");
    let pub_str = std::str::from_utf8(&pub_key_decoded).expect("pubkey decode utf8");
    let public_key = minisign_verify::PublicKey::decode(pub_str)
        .expect("pubkey must be a valid minisign public key");

    let signature_b64_decoded = base64::engine::general_purpose::STANDARD
        .decode(&signature_b64)
        .expect("signature must be base64");
    let sig_str = std::str::from_utf8(&signature_b64_decoded).expect("signature decode utf8");
    let signature = minisign_verify::Signature::decode(sig_str)
        .expect("signature must be a valid minisign signature");

    let data = std::fs::read(&dmg).expect("read artifact");
    public_key
        .verify(&data, &signature, true)
        .expect("signature MUST verify against the configured public key");

    // also sanity: tampered data must fail
    let mut tampered = data.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 0xFF;
    let result = public_key.verify(&tampered, &signature, true);
    assert!(result.is_err(), "tampered artifact must NOT verify");
}
