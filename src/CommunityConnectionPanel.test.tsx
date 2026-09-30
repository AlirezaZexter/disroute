import "@testing-library/jest-dom/vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { CommunityConnectionPanel, type SourceDraft } from "./components/CommunityConnectionPanel";
import type { CommunitySnapshot } from "./types";

afterEach(cleanup);
const snapshot: CommunitySnapshot = {
  sources: [{ id: "tr", name: "Turkey", attribution: "Test operator", location: "https://example.com", kind: "subscription", enabled: true, redistributionAuthorized: true, refreshIntervalMinutes: 60, timeoutSeconds: 12, lastError: "دریافت منبع بیش از حد طول کشید." }],
  candidates: [], stale: true, freshSourceCount: 7, cachedSourceCount: 1,
  acknowledgedWarning: true, automaticFailover: true,
};
function panel(community: CommunitySnapshot) {
  return render(<CommunityConnectionPanel community={community} results={[]} busy="" error="" warningChecked sourceDraft={{} as SourceDraft} connected={false}
    onSubmit={vi.fn()} onWarningChecked={vi.fn()} onAcknowledge={vi.fn()} onRefresh={vi.fn()} onScan={vi.fn()} onCancelScan={vi.fn()}
    onFailoverChange={vi.fn()} onReplaceSources={vi.fn()} onSourceDraftChange={vi.fn()} onAddSource={vi.fn()} onClearData={vi.fn()} />);
}
it("distinguishes mixed fresh/cached sources and exposes errors outside collapsed management", () => {
  const { container } = panel(snapshot);
  expect(screen.queryByText("فهرست قدیمی")).not.toBeInTheDocument();
  expect(screen.getByText("بخشی از منابع ذخیره‌شده است")).toBeInTheDocument();
  expect(screen.getByText(/۷ منبع به‌روز/)).toBeInTheDocument();
  expect(container.querySelector(".source-refresh-status")).toHaveTextContent("Turkey");
  expect(container.querySelector(".source-refresh-status")).toHaveTextContent("دریافت منبع بیش از حد طول کشید.");
});
it("explicitly explains full cached fallback", () => {
  panel({ ...snapshot, freshSourceCount: 0 });
  expect(screen.getByText("استفاده از فهرست ذخیره‌شده")).toBeInTheDocument();
  expect(screen.getByText(/دریافت فهرست تازه موفق نبود/)).toBeInTheDocument();
});
it("does not show disabled source failures", () => {
  const { container } = panel({ ...snapshot, sources: snapshot.sources.map(source => ({ ...source, enabled: false })), stale: false, freshSourceCount: 0, cachedSourceCount: 0 });
  expect(container.querySelector(".source-refresh-status")).toBeNull();
});
it("does not imply a usable cached list exists when all sources failed before first fetch", () => {
  panel({ ...snapshot, stale: false, freshSourceCount: 0, cachedSourceCount: 0 });
  expect(screen.getByText("فهرستی دریافت نشد. خطای منابع را بررسی و دوباره نوسازی کنید.")).toBeInTheDocument();
  expect(screen.queryByText("استفاده از فهرست ذخیره‌شده")).not.toBeInTheDocument();
});
