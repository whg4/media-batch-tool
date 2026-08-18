import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const fixtureDir = path.resolve(here, "fixtures");

const FIXTURES = ["photo1.png", "photo2.jpg", "video1.mp4"].map((f) =>
  path.join(fixtureDir, f),
);

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
async function seedFiles() {
  await browser.tauri.execute(
    (_tauri: unknown, args: { paths: string[] }) => {
      const add = (window as unknown as { __MBT_ADD_PATHS__?: (p: string[]) => Promise<unknown> })
        .__MBT_ADD_PATHS__;
      if (typeof add !== "function") {
        throw new Error("__MBT_ADD_PATHS__ hook missing — need a DEV-mode frontend build");
      }
      return add(args.paths);
    },
    { paths: FIXTURES },
  );
}

describe("Real app E2E (embedded WebDriver)", () => {
  it("runs the full 智能瘦身 flow against the real binary", async () => {
    // 1. Home: app name + mode cards
    await waitForText("媒体批处理工具", 30000);
    await clickButtonContaining("智能瘦身");

    // 2. Drop view: seed real files through the dev hook (native dialog is not automatable)
    await waitForText("下一步");
    await seedFiles();

    for (const name of ["photo1.png", "photo2.jpg", "video1.mp4"]) {
      await waitForText(name);
    }
    await clickButtonContaining("下一步");

    // 3. Template view: slim template auto-selected; start processing
    await waitForText("选择处理方式");
    await clickButtonContaining("开始处理");

    // 4. Processing → Done
    await waitForText("处理中", 20000);
    await waitForText("处理完成", 120000);

    // 5. Done view: savings line + per-status rows (the two tiny images are
    // smart-skipped because they can't shrink; the video is processed).
    await waitForText("共节省", 10000);
    await waitForText("处理后", 10000);
    await waitForText("已跳过", 10000);
    await waitForText("video1.mp4", 10000);
  });

  it("exposes the WDIO execute API from the real app", async () => {
    const url = await browser.tauri.execute(() => window.location.href);
    expect(url.startsWith("tauri://localhost")).toBe(true);
  });
});
