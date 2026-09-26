import { describe, expect, it, vi } from "vitest";
import type { Update, DownloadEvent } from "@tauri-apps/plugin-updater";
import { downloadAndInstall } from "./updater";

describe("updater download", () => {
  it("coalesces chunk updates but always reports completion before installation", async () => {
    const progress = vi.fn();
    const beforeInstall = vi.fn();
    const install = vi.fn();
    const update = {
      download: async (report: (event: DownloadEvent) => void) => {
        report({ event: "Started", data: { contentLength: 1000 } });
        for (let i = 0; i < 1000; i++) report({ event: "Progress", data: { chunkLength: 1 } });
        report({ event: "Finished" });
      }, install,
    } as unknown as Update;
    await downloadAndInstall(update, progress, beforeInstall);
    expect(progress.mock.calls.length).toBeLessThan(10);
    expect(progress).toHaveBeenLastCalledWith(expect.objectContaining({ downloaded: 1000, percent: 100 }));
    expect(beforeInstall.mock.invocationCallOrder[0]).toBeLessThan(install.mock.invocationCallOrder[0]);
  });

  it("never stops the working connection or installs after a failed download", async () => {
    const beforeInstall = vi.fn();
    const install = vi.fn();
    const update = { download: async () => { throw new Error("offline"); }, install } as unknown as Update;
    await expect(downloadAndInstall(update, vi.fn(), beforeInstall)).rejects.toThrow("offline");
    expect(beforeInstall).not.toHaveBeenCalled();
    expect(install).not.toHaveBeenCalled();
  });
});
