import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const fixtureDir = path.resolve(here, "fixtures");

const PNG = path.join(fixtureDir, "photo1.png");
const JPG = path.join(fixtureDir, "photo2.jpg");
const MP4 = path.join(fixtureDir, "video1.mp4");

/** Poll the real app DOM (via execute) until `text` appears. */
async function waitForText(text: string, timeout = 20000) {
  await browser.waitUntil(
    async () =>
      browser.tauri.execute(
        (_t: unknown, args: { text: string }) =>
          document.body.innerText.includes(args.text),
        { text },
      ),
    { timeout, timeoutMsg: `"${text}" never appeared in the app DOM` },
  );
}

/** Click the first <button> whose visible text contains `text`. */
async function clickButtonContaining(text: string) {
  const btn = await browser.$(`//button[contains(normalize-space(.), "${text}")]`);
  await btn.waitForDisplayed({ timeout: 15000 });
  await btn.click();
}

/** Seed real fixture files through the dev-only store hook. */
async function seedFiles(paths: string[]) {
  await browser.tauri.execute(
    (_tauri: unknown, args: { paths: string[] }) => {
      const add = (window as unknown as { __MBT_ADD_PATHS__?: (p: string[]) => Promise<unknown> })
        .__MBT_ADD_PATHS__;
      if (typeof add !== "function") {
        throw new Error("__MBT_ADD_PATHS__ hook missing — need a DEV-mode frontend build");
      }
      return add(args.paths);
    },
    { paths },
  );
}

/**
 * Drive one full mode flow: pick mode → drop files → (optional template) →
 * start → Done page.
 */
async function runModeFlow(
  modeCard: string,
  files: string[],
  templateButton?: string,
) {
  await clickButtonContaining(modeCard);
  await waitForText("下一步");
  await seedFiles(files);
  for (const f of files) await waitForText(path.basename(f));
  await clickButtonContaining("下一步");
  await waitForText("选择处理方式");
  if (templateButton) await clickButtonContaining(templateButton);
  await clickButtonContaining("开始处理");
  await waitForText("处理完成", 120000);
  await waitForText("共节省", 10000);
}

/** From the Done page, go back home for another round. */
async function backHome() {
  await clickButtonContaining("继续处理");
  await waitForText("媒体批处理工具");
}

describe("Real app E2E (embedded WebDriver)", () => {
  it("智能瘦身: processes the video, smart-skips tiny images, exports", async () => {
    await waitForText("媒体批处理工具", 30000);
    await runModeFlow("智能瘦身", [PNG, JPG, MP4]);

    // the video is re-encoded; the two tiny images are smart-skipped
    await waitForText("处理后", 10000);
    await waitForText("已跳过", 10000);
    await waitForText("video1.mp4", 10000);
    // NOTE: the export button opens a native folder picker, which WebDriver
    // can't automate (window.__TAURI_INTERNALS__.invoke is non-writable, and
    // the wdio mock only wraps __TAURI__.core.invoke which @tauri-apps/api
    // doesn't use). Export is covered by the Rust test
    // `export_files_copies_processed_outputs_and_dedups` (real outputs →
    // real copy) and by the Playwright flow test (UI handler, mocked dialog).
  });

  it("格式转换: converts a PNG to WebP via the 转为 WebP template", async () => {
    await backHome();
    await runModeFlow("格式转换", [PNG], "转为 WebP");

    // converted — not skipped — so the processed badge is present
    await waitForText("处理后", 10000);
    await waitForText("photo1.png", 10000);
  });

  it("社交媒体适配: adapts an image via the 朋友圈 template", async () => {
    await backHome();
    await runModeFlow("社交媒体适配", [PNG], "朋友圈");

    await waitForText("处理后", 10000);
    await waitForText("photo1.png", 10000);
  });

  it("exposes the WDIO execute API from the real app", async () => {
    const url = await browser.tauri.execute(() => window.location.href);
    expect(url.startsWith("tauri://localhost")).toBe(true);
  });
});
