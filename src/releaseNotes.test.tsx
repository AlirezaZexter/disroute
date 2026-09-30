import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ReleasePreview, WhatsNewNotice, WhatsNewView } from "./components/WhatsNew";
import { APP_VERSION, currentRelease, hasUnreadRelease, markReleaseRead, RELEASE_SEEN_KEY } from "./releaseNotes";

beforeEach(() => localStorage.clear());
afterEach(() => { cleanup(); vi.restoreAllMocks(); });

describe("release notes", () => {
  it("bundles notes for the running version, available offline", () => {
    expect(currentRelease.version).toBe(APP_VERSION);
    render(<WhatsNewView />);
    expect(screen.getByRole("heading", { name: "تازه‌های نسخه" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "تازه‌های نسخه" })).toHaveFocus();
    expect(screen.getByText(APP_VERSION)).toHaveAttribute("dir", "ltr");
    expect(screen.getByRole("heading", { name: "لوگوی کامل داخل برنامه" })).toBeInTheDocument();
  });
  it("remembers read/dismissed notes without storing profile data", () => {
    expect(hasUnreadRelease()).toBe(true);
    markReleaseRead();
    expect(localStorage.getItem(RELEASE_SEEN_KEY)).toBe(APP_VERSION);
    expect(hasUnreadRelease()).toBe(false);
    localStorage.setItem(RELEASE_SEEN_KEY, "0.5.5");
    expect(hasUnreadRelease()).toBe(true);
    expect(localStorage.length).toBe(1);
  });
  it("never breaks startup when browser storage is blocked", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => { throw new Error("denied"); });
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("denied"); });
    expect(hasUnreadRelease()).toBe(true);
    expect(() => markReleaseRead()).not.toThrow();
  });
  it("offers explicit open and dismiss controls, not a modal", () => {
    const open = vi.fn(), dismiss = vi.fn();
    render(<WhatsNewNotice onOpen={open} onDismiss={dismiss} />);
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "دیدن تغییرات" }));
    fireEvent.click(screen.getByRole("button", { name: "بستن اعلان تازه‌های نسخه" }));
    expect(open).toHaveBeenCalledOnce();
    expect(dismiss).toHaveBeenCalledOnce();
  });
  it("renders remote updater notes as bounded text, never HTML", () => {
    const { container } = render(<ReleasePreview notes={'## Changes\n- <img src=x onerror=alert(1)>\n' + 'a'.repeat(100000)} />);
    expect(container.querySelector("img")).toBeNull();
    expect(container.textContent).toContain("<img src=x onerror=alert(1)>");
    expect(container.textContent!.length).toBeLessThan(17000);
    expect(screen.getByText("Changes")).toBeInTheDocument();
  });
  it("groups update headings and bullet lists with stable Persian direction", () => {
    const { container } = render(<ReleasePreview notes={'## تغییرات\nDiscord بهتر نمایش داده می‌شود.\n\n- اصلاح لوگو\n- نظم توضیحات\n\n## قبل از نصب\nاتصال قطع می‌شود.'} />);
    expect(screen.getByRole("heading", { name: "تغییرات" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "قبل از نصب" })).toBeInTheDocument();
    expect(screen.getAllByRole("listitem")).toHaveLength(2);
    expect(container.querySelector(".release-preview")).toHaveAttribute("dir", "rtl");
    expect(screen.getByText("Discord")).toHaveAttribute("dir", "ltr");
    expect(container.querySelector("p")).not.toHaveAttribute("dir", "auto");
  });
});
