import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ProgressBar from "../components/ProgressBar.vue";

describe("ProgressBar", () => {
  it("clamps width to 100", () => {
    const w = mount(ProgressBar, { props: { percent: 150 } });
    expect(w.find(".bg-gradient-to-r").attributes("style")).toContain("100%");
  });
  it("renders partial progress", () => {
    const w = mount(ProgressBar, { props: { percent: 40 } });
    expect(w.find(".bg-gradient-to-r").attributes("style")).toContain("40%");
  });
});
