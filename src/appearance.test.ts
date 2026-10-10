import { beforeEach, describe, expect, it, vi } from "vitest";
import { readAppearance, saveAppearance } from "./appearance";

beforeEach(() => { vi.restoreAllMocks(); localStorage.clear(); });

describe("appearance preference", () => {
  it("defaults to light and accepts only supported values", () => {
    expect(readAppearance()).toBe("light");
    localStorage.setItem("disroute.ui.appearance", "unexpected");
    expect(readAppearance()).toBe("light");
    saveAppearance("dark");
    expect(readAppearance()).toBe("dark");
  });
  it("does not break the application when preference storage is unavailable", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => { throw new Error("blocked"); });
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("blocked"); });
    expect(readAppearance()).toBe("light");
    expect(() => saveAppearance("dark")).not.toThrow();
  });
});
