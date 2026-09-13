import { invoke } from "@tauri-apps/api/core";
import type { AppStatus, ProxyProfile } from "./types";

const browserFallback: AppStatus = {
  status: "disconnected",
  engineReady: false,
  isElevated: false,
  message: "پیش‌نمایش رابط؛ موتور فقط داخل برنامهٔ ویندوز اجرا می‌شود.",
};

const isTauri = () => "__TAURI_INTERNALS__" in window;

export async function getStatus(): Promise<AppStatus> {
  return isTauri() ? invoke<AppStatus>("get_status") : browserFallback;
}

export async function saveProfile(profile: ProxyProfile): Promise<void> {
  if (isTauri()) await invoke("save_profile", { profile });
}

export async function connect(profile: ProxyProfile): Promise<AppStatus> {
  if (!isTauri()) return { ...browserFallback, status: "error" };
  return invoke<AppStatus>("start_tunnel", { profile });
}

export async function disconnect(): Promise<AppStatus> {
  if (!isTauri()) return browserFallback;
  return invoke<AppStatus>("stop_tunnel");
}

