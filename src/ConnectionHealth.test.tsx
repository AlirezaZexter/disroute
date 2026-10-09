import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ConnectionHealthPanel } from "./components/ConnectionHealthPanel";
import { ConnectionStatus } from "./components/ConnectionStatus";
import type { ConnectionHealth } from "./types";

afterEach(cleanup);
const snapshot: ConnectionHealth = {
  phase: "healthy", mode: "personal", connectedAt: Date.now() / 1000 - 90,
  checkedAt: Date.now() / 1000, latencyMs: 180, consecutiveFailures: 0, recoveries: 0,
  samples: [{ checkedAt: Date.now() / 1000, latencyMs: 180 }],
};

describe("live connection health", () => {
  it("does not invent latency or connection success before a real sample", () => {
    render(<ConnectionHealthPanel health={null} connected={false} busy={false} error="" onCheck={() => {}} />);
    expect(screen.getByText("هنوز اندازه‌گیری نشده")).toBeInTheDocument();
    expect(screen.queryByText("Discord پاسخ می‌دهد")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "بررسی همین حالا" })).toBeDisabled();
  });
  it("shows measured history, and checks only when requested", () => {
    const check = vi.fn();
    render(<ConnectionHealthPanel health={snapshot} connected busy={false} error="" onCheck={check} />);
    expect(screen.getByText("Discord پاسخ می‌دهد")).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "۱ بررسی موفق از ۱ بررسی اخیر" })).toBeInTheDocument();
    expect(check).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "بررسی همین حالا" }));
    expect(check).toHaveBeenCalledOnce();
  });
  it("does not present a stale success as current connectivity", () => {
    render(<ConnectionHealthPanel health={{ ...snapshot, checkedAt: Date.now() / 1000 - 60 }} connected busy={false} error="" onCheck={() => {}} />);
    expect(screen.queryByText("Discord پاسخ می‌دهد")).not.toBeInTheDocument();
    expect(screen.getByText(/بررسی قبلی قدیمی شده/)).toBeInTheDocument();
  });
  it("uses text and failure marks when the engine runs but Discord is unreachable", () => {
    const failed: ConnectionHealth = { ...snapshot, phase: "unavailable", latencyMs: null, consecutiveFailures: 3, samples: [{ checkedAt: Date.now() / 1000, latencyMs: null }] };
    render(<><ConnectionHealthPanel health={failed} connected busy={false} error="" onCheck={() => {}} />
      <ConnectionStatus health={failed} appStatus={{ status: "connected", engineReady: true, isElevated: true, message: "engine running" }} mode="personal"
        busy={false} discordBusy={false} communityBusy="" connectDisabled={false} inGuide={false} onOpenConnection={() => {}} onRestartDiscord={() => {}} onDisconnect={() => {}} /></>);
    expect(screen.getByText("اتصال نیاز به بررسی دارد")).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "۰ بررسی موفق از ۱ بررسی اخیر" })).toBeInTheDocument();
    expect(screen.getByText("×")).toBeInTheDocument();
  });
});
