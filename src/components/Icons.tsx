export function ShieldIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 2 4.5 5v5.7c0 4.8 3 9.2 7.5 11.3 4.5-2.1 7.5-6.5 7.5-11.3V5L12 2Z" />
      <path d="m8.8 12 2 2 4.4-4.5" />
    </svg>
  );
}

export function SpinnerIcon() {
  return (
    <svg className="spinner-icon" viewBox="0 0 20 20" aria-hidden="true">
      <circle cx="10" cy="10" r="7" />
      <path d="M10 3a7 7 0 0 1 6.6 4.7" />
    </svg>
  );
}

export type InterfaceIconName = "connection" | "guide" | "whatsNew" | "sun" | "moon" | "update" | "tray";

export function InterfaceIcon({ name }: { name: InterfaceIconName }) {
  const paths: Record<InterfaceIconName, ReactNode> = {
    connection: <><path d="M8.5 7.5h7v9h-7zM10 3v4.5M14 3v4.5M12 16.5V21" /><path d="M5 10h3.5M15.5 10H19" /></>,
    guide: <><path d="M12 5c-3-2-6-2-9-1v14c3-1 6-1 9 1 3-2 6-2 9-1V4c-3-1-6-1-9 1ZM12 5v14" /><path d="M6 8h3M6 11h3M15 8h3M15 11h3" /></>,
    whatsNew: <><path d="m12 3 2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5Z" /></>,
    sun: <><circle cx="12" cy="12" r="4" /><path d="M12 2v2M12 20v2M2 12h2M20 12h2M5 5l1.4 1.4M17.6 17.6 19 19M5 19l1.4-1.4M17.6 6.4 19 5" /></>,
    moon: <path d="M20.5 14.3A9 9 0 0 1 9.7 3.5a9 9 0 1 0 10.8 10.8Z" />,
    update: <><path d="M20 7v5h-5M4 17v-5h5" /><path d="M5.4 7a8 8 0 0 1 13-1l1.6 2M4 16l1.6 2a8 8 0 0 0 13-1" /></>,
    tray: <><path d="M4 16v4h16v-4M12 3v12M7 10l5 5 5-5" /></>,
  };
  return <svg className="interface-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.65" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">{paths[name]}</svg>;
}
import type { ReactNode } from "react";
