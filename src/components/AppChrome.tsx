import { motion } from "motion/react";
import type { UpdateProgress } from "../updater";
import { collapseVariants, transitions } from "../motion";
import { SpinnerIcon } from "./Icons";
import brandIcon from "../../src-tauri/icons/128x128.png";

export type AppView = "connection" | "guide";
export type UpdatePhase = "idle" | "checking" | "available" | "current" | "downloading" | "installing" | "error";

interface TopBarProps {
  updatePhase: UpdatePhase;
  onCheckUpdates: () => void;
  onHideToTray: () => void;
}

export function TopBar({ updatePhase, onCheckUpdates, onHideToTray }: TopBarProps) {
  const checking = updatePhase === "checking";
  const updating = updatePhase === "downloading" || updatePhase === "installing";

  return (
    <header className="topbar">
      <div className="brand-lockup">
        <div className="brand-mark"><img src={brandIcon} alt="" width="42" height="42" /></div>
        <div>
          <h1>DisRoute</h1>
          <p>مسیر مستقل Discord</p>
        </div>
      </div>
      <div className="header-actions">
        <span className="version" dir="ltr">v0.5.4</span>
        <button
          className="text-button update-check-button"
          type="button"
          disabled={checking || updating}
          aria-busy={checking || updating}
          onClick={onCheckUpdates}
        >
          {(checking || updating) && <SpinnerIcon />}
          <span>{checking ? "در حال بررسی…" : updating ? "در حال آپدیت…" : "بررسی آپدیت"}</span>
        </button>
        <button
          className="text-button tray-button"
          type="button"
          title="پنجره بسته می‌شود و برنامه در System tray فعال می‌ماند"
          onClick={onHideToTray}
        >
          Minimize to system tray
        </button>
      </div>
    </header>
  );
}

interface UpdateNoticeProps {
  phase: UpdatePhase;
  info: { version: string; notes?: string } | null;
  progress: UpdateProgress;
  error: string;
  onInstall: () => void;
  onClose: () => void;
}

export function UpdateNotice({ phase, info, progress, error, onInstall, onClose }: UpdateNoticeProps) {
  return (
    <motion.section
      className={`update-card update-${phase}`}
      variants={collapseVariants}
      initial="initial"
      animate="enter"
      exit="exit"
      role={phase === "error" ? "alert" : "status"}
      aria-live={phase === "error" ? "assertive" : "polite"}
      aria-busy={phase === "downloading" || phase === "installing"}
    >
      <div className="update-copy">
        <span className="section-label">آپدیت DisRoute</span>
        {phase === "available" && <><strong>نسخه <bdi dir="ltr">{info?.version}</bdi> آماده است</strong><p>{info?.notes || "نسخهٔ جدید از GitHub دانلود و پس از بررسی امضا نصب می‌شود."}</p></>}
        {phase === "current" && <><strong>نسخه جدیدی منتشر نشده</strong><p>همین نسخه، آخرین نسخهٔ موجود است.</p></>}
        {phase === "downloading" && <><strong>در حال دانلود نسخهٔ <bdi dir="ltr">{info?.version}</bdi></strong><p>{progress.percent === undefined ? "در حال دریافت فایل…" : `${progress.percent.toLocaleString("fa-IR")}٪ دریافت شده`} · <bdi dir="ltr">{(progress.downloaded / 1048576).toFixed(1)}{progress.total ? ` / ${(progress.total / 1048576).toFixed(1)}` : ""} MB</bdi>{!!progress.bytesPerSecond && <> · <bdi dir="ltr">{(progress.bytesPerSecond / 1024).toFixed(0)} KB/s</bdi></>}</p><p>آپدیت، فایل کامل نصب را دریافت می‌کند؛ فقط تغییرات دانلود نمی‌شوند.</p></>}
        {phase === "installing" && <><strong>در حال نصب</strong><p>پس از پایان نصب، DisRoute دوباره اجرا می‌شود.</p></>}
        {phase === "error" && <><strong>آپدیت انجام نشد</strong><p dir="auto">{error}</p></>}
      </div>
      {phase === "downloading" && (
        <div className="update-progress" role="progressbar" aria-label="پیشرفت دانلود آپدیت" aria-valuemin={0} aria-valuemax={100} aria-valuenow={progress.percent}>
          <motion.span animate={{ scaleX: (progress.percent || 0) / 100 }} transition={transitions.micro} />
        </div>
      )}
      <div className="update-actions">
        {phase === "available" && <button className="button button-primary" type="button" onClick={onInstall}>دانلود و نصب</button>}
        {(phase === "available" || phase === "current" || phase === "error") && <button className="text-button" type="button" onClick={onClose}>{phase === "available" ? "بعداً" : "بستن"}</button>}
      </div>
    </motion.section>
  );
}

export function WorkspaceHeading() {
  return (
    <div className="workspace-heading">
      <div>
        <h2>کنترل اتصال</h2>
        <p>Discord از مسیر پروکسی عبور می‌کند؛ بقیهٔ برنامه‌ها مستقیم می‌مانند.</p>
      </div>
      <span className="local-badge"><i /> مسیریابی انتخابی</span>
    </div>
  );
}

export function ViewSwitch({ view, onChange }: { view: AppView; onChange: (view: AppView) => void }) {
  return (
    <nav className="view-switch" aria-label="بخش‌های برنامه">
      {([ ["connection", "اتصال"], ["guide", "راهنمای شروع"] ] as const).map(([id, label]) => (
        <button type="button" key={id} aria-pressed={view === id} onClick={() => onChange(id)}>
          {view === id && <motion.span className="selected-view" layoutId="selected-view" transition={transitions.layout} />}
          <span>{label}</span>
        </button>
      ))}
    </nav>
  );
}

export function AppFooter() {
  return (
    <footer className="app-footer">
      <span>بستن پنجره: <bdi dir="ltr">Minimize to system tray</bdi> · خروج کامل: منوی کنار ساعت</span>
      <span className="creator-credit">Created by Zexter</span>
      <span dir="ltr">VLESS · VMess · Trojan · SS</span>
    </footer>
  );
}
