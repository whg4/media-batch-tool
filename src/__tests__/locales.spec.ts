import { describe, expect, it } from "vitest";
import { setLocale, t } from "../locales";

describe("i18n", () => {
  it("defaults to Chinese", () => {
    expect(t("app_name")).toContain("媒体");
  });
  it("switches to English", () => {
    setLocale("en");
    expect(t("app_name")).toBe("Media Batch Tool");
    setLocale("zh");
  });
  it("interpolates variables", () => {
    expect(t("saved_total", { size: "2.3 GB" })).toContain("2.3 GB");
  });
});
