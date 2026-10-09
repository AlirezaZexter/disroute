import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import * as api from "./api";
import { useDisRouteController } from "./useDisRouteController";

vi.mock("./api", async (original) => ({ ...await original<typeof import("./api")>(), getStatus: vi.fn(), getConnectionHealth: vi.fn(), checkPrerequisites: vi.fn() }));
const status = { status: "disconnected" as const, engineReady: true, isElevated: true, message: "ready" };
beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(api.getStatus).mockResolvedValue(status);
  vi.mocked(api.getConnectionHealth).mockResolvedValue(null);
  vi.mocked(api.checkPrerequisites).mockResolvedValue(null);
});
afterEach(() => { cleanup(); vi.useRealTimers(); vi.resetAllMocks(); });

it("observes a background recovery even after the UI saw a disconnected engine", async () => {
  const { result } = renderHook(useDisRouteController);
  await act(async () => { await vi.advanceTimersByTimeAsync(1000); });
  expect(result.current.connected).toBe(false);
  vi.mocked(api.getStatus).mockResolvedValue({ ...status, status: "connected" });
  await act(async () => { await vi.advanceTimersByTimeAsync(5000); });
  expect(result.current.connected).toBe(true);
});

it("does not overlap slow polls or update after the controller unmounts", async () => {
  const { unmount } = renderHook(useDisRouteController);
  await act(async () => { await vi.advanceTimersByTimeAsync(1000); });
  const before = vi.mocked(api.getStatus).mock.calls.length;
  vi.mocked(api.getStatus).mockReturnValue(new Promise(() => {}));
  await act(async () => { await vi.advanceTimersByTimeAsync(20000); });
  expect(vi.mocked(api.getStatus).mock.calls.length).toBe(before + 1);
  unmount();
  await act(async () => { await vi.advanceTimersByTimeAsync(20000); });
  expect(vi.mocked(api.getStatus).mock.calls.length).toBe(before + 1);
});
