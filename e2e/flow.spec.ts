import { test, expect } from "@playwright/test";
import { mockTauriInit } from "./mock-tauri";

test("main flow: home → drop → template → processing → done", async ({ page }) => {
  await page.addInitScript(mockTauriInit);
  await page.goto("http://localhost:1420");

  // 1. home
  await expect(page.getByText("媒体批处理工具").first()).toBeVisible();

  // 2. pick 智能瘦身 mode
  await page.getByText("智能瘦身", { exact: true }).first().click();
  await expect(page.getByText("下一步")).toBeVisible();

  // 3. add files via the mocked dialog
  await page.getByRole("button", { name: "选择文件", exact: true }).click();
  await expect(page.locator("text=photo1.jpg")).toBeVisible({ timeout: 5000 });
  await expect(page.locator("text=video1.mp4")).toBeVisible();

  // 4. next → template
  await page.getByRole("button", { name: /下一步/ }).click();
  await expect(page.getByText("智能瘦身", { exact: true }).last()).toBeVisible();

  // 5. start processing
  await page.getByRole("button", { name: /开始处理/ }).click();
  await expect(page.getByText("处理中", { exact: true })).toBeVisible({ timeout: 5000 });

  // 6. done page
  await expect(page.getByText("处理完成").first()).toBeVisible({ timeout: 8000 });
  await expect(page.getByText(/共节省/)).toBeVisible();

  // 7. export
  await page.getByRole("button", { name: /导出到/ }).click();
  await expect(page.getByText(/已导出/)).toBeVisible({ timeout: 5000 });
});
