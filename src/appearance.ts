export type Appearance = "light" | "dark";
const preferenceKey = "disroute.ui.appearance";

export function readAppearance(): Appearance {
  try { return localStorage.getItem(preferenceKey) === "dark" ? "dark" : "light"; }
  catch { return "light"; }
}

export function saveAppearance(appearance: Appearance): void {
  try { localStorage.setItem(preferenceKey, appearance); }
  catch { /* A cosmetic preference must not prevent connection or navigation. */ }
}
