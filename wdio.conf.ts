import type { Options } from "@wdio/types";

// Real-app E2E via WebdriverIO's Tauri service with the embedded WebDriver
// (runs the W3C server inside the app — no external tauri-driver needed).
// Requires a RELEASE build of the app with the `wdio` feature and a DEV-mode
// frontend build (so the frontend wdio plugin + test hook are bundled):
//
//   pnpm e2e:real:build   # vite build --mode development + cargo build --release --features wdio
//   pnpm test:e2e:real    # this config
export const config: WebdriverIO.Config = {
  runner: "local",
  specs: ["./e2e/real-app.spec.ts"],
  maxInstances: 1,

  capabilities: [
    {
      browserName: "tauri",
      "tauri:options": {
        application: "./src-tauri/target/release/media-batch-tool",
      },
    },
  ],

  services: [
    [
      "tauri",
      {
        driverProvider: "embedded",
        startTimeout: 60000,
        captureBackendLogs: true,
        captureFrontendLogs: true,
      },
    ],
  ],

  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 180000,
  },
  reporters: ["spec"],
  logLevel: "info",
  waitforTimeout: 15000,
  autoCompileOpts: {
    autoCompile: true,
    tsNodeOpts: {
      transpileOnly: true,
    },
  },
};
