import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";

export interface UpdateProgress {
  downloaded: number;
  total?: number;
  percent?: number;
}

const isTauri = () => "__TAURI_INTERNALS__" in window;

export async function findUpdate(): Promise<Update | null> {
  if (!isTauri()) return null;
  return check({ timeout: 20_000 });
}

export async function downloadAndInstall(
  update: Update,
  onProgress: (progress: UpdateProgress) => void,
  beforeInstall: () => Promise<void>,
): Promise<void> {
  let downloaded = 0;
  let total: number | undefined;

  const report = () => {
    const percent = total && total > 0 ? Math.min(100, Math.round((downloaded / total) * 100)) : undefined;
    onProgress({ downloaded, total, percent });
  };

  await update.download((event: DownloadEvent) => {
    if (event.event === "Started") {
      total = event.data.contentLength;
      report();
    } else if (event.event === "Progress") {
      downloaded += event.data.chunkLength;
      report();
    } else {
      if (total) downloaded = total;
      report();
    }
  });
  await beforeInstall();
  await update.install();
}
