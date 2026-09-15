import { invoke } from "@tauri-apps/api/core";
import type { AppStatus, ProxyProfile } from "./types";

const browserFallback: AppStatus = {
  status: "disconnected",
  engineReady: false,
  isElevated: false,
  message: "پیش‌نمایش رابط؛ موتور فقط داخل برنامهٔ ویندوز اجرا می‌شود.",
};

const isTauri = () => "__TAURI_INTERNALS__" in window;

export async function hideToTray(): Promise<void> {
  if (!isTauri()) throw new Error("Minimize to tray فقط در نسخهٔ ویندوز در دسترس است.");
  await invoke("hide_to_tray");
}

export async function getStatus(): Promise<AppStatus> {
  return isTauri() ? invoke<AppStatus>("get_status") : browserFallback;
}

export async function saveProfile(profile: ProxyProfile): Promise<void> {
  if (!isTauri()) throw new Error("ذخیره امن فقط در برنامهٔ ویندوز در دسترس است.");
  await invoke("save_profile", { profile });
}

export async function loadProfile(): Promise<ProxyProfile | null> {
  return isTauri() ? invoke("load_profile") : null;
}
export async function forgetProfile(): Promise<void> {
  if (isTauri()) await invoke("forget_profile");
}

export async function connect(profile: ProxyProfile): Promise<AppStatus> {
  if (!isTauri()) return { ...browserFallback, status: "error" };
  return invoke<AppStatus>("start_tunnel", { profile });
}

export async function disconnect(): Promise<AppStatus> {
  if (!isTauri()) return browserFallback;
  return invoke<AppStatus>("stop_tunnel");
}

export async function restartDiscord(): Promise<string> {
  if (!isTauri()) throw new Error("Restart Discord فقط در نسخهٔ ویندوز در دسترس است.");
  return invoke<string>("restart_discord");
}
