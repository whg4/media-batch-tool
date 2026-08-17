import { describe, expect, it } from "vitest";
import { formatBytes, formatDuration } from "../lib/format";

describe("formatBytes", () => {
  it("formats zero", () => {
    expect(formatBytes(0)).toBe("0 B");
  });
  it("formats KB/MB/GB", () => {
    expect(formatBytes(1024)).toBe("1.0 KB");
    expect(formatBytes(5 * 1024 * 1024)).toBe("5.0 MB");
    expect(formatBytes(2 * 1024 * 1024 * 1024)).toBe("2.0 GB");
  });
  it("handles null", () => {
    expect(formatBytes(null)).toBe("--");
  });
});

describe("formatDuration", () => {
  it("formats mm:ss", () => {
    expect(formatDuration(65)).toBe("1:05");
    expect(formatDuration(3600)).toBe("60:00");
  });
  it("handles missing", () => {
    expect(formatDuration(null)).toBe("--");
  });
});
