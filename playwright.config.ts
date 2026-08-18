import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  // WDIO real-app spec + fixtures live in e2e/ but belong to a different runner
  testIgnore: ["**/real-app.spec.ts", "**/fixtures/**"],
  timeout: 60_000,
  use: {
    channel: "chrome",
    headless: true,
  },
});
