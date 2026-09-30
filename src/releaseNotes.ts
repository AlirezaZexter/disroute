import manifest from "./release-notes.json";
import { version } from "../package.json";

export const APP_VERSION = version;
export const RELEASE_SEEN_KEY = "disroute.release-notes.seen";
export const currentRelease = manifest.releases.find((release) => release.version === APP_VERSION)!;

// Cosmetic state only. Blocked storage must never stop a connection or startup.
export function hasUnreadRelease(): boolean {
  try { return localStorage.getItem(RELEASE_SEEN_KEY) !== APP_VERSION; }
  catch { return true; }
}
export function markReleaseRead(): void {
  try { localStorage.setItem(RELEASE_SEEN_KEY, APP_VERSION); }
  catch { /* The in-memory dismissal still works. */ }
}
