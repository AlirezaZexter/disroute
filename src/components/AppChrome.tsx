import { motion } from "motion/react";
import type { UpdateProgress } from "../updater";
import { collapseVariants, transitions } from "../motion";
import { InterfaceIcon, SpinnerIcon } from "./Icons";
import type { Appearance } from "../appearance";
import { APP_VERSION } from "../releaseNotes";
import { ReleasePreview } from "./WhatsNew";

export type AppView = "connection" | "guide" | "whatsNew";
export type UpdatePhase = "idle" | "checking" | "available" | "current" | "downloading" | "installing" | "error";

interface TopBarProps {
  updatePhase: UpdatePhase;
  onCheckUpdates: () => void;
  onHideToTray: () => void;
  appearance?: Appearance;
  onToggleAppearance?: () => void;
}

export function TopBar({ updatePhase, onCheckUpdates, onHideToTray, appearance = "light", onToggleAppearance }: TopBarProps) {
  const checking = updatePhase === "checking";
  const updating = updatePhase === "downloading" || updatePhase === "installing";

  return (
    <header className="topbar">
      <div className="brand-lockup">
        <div>
          <h1>DisRoute</h1>
          <p dir="ltr">Discord connection utility</p>
        </div>
      </div>
      <div className="header-actions">
        <span className="version" dir="ltr">v{APP_VERSION}</span>
        <button
          className="text-button update-check-button"
          type="button"
          disabled={checking || updating}
          aria-label={checking ? "در حال بررسی…" : updating ? "در حال آپدیت…" : "بررسی آپدیت"}
          aria-busy={checking || updating}
          onClick={onCheckUpdates}
        >
          {checking || updating ? <SpinnerIcon /> : <InterfaceIcon name="update" />}
          <span>{checking ? "در حال بررسی…" : updating ? "در حال آپدیت…" : "بررسی آپدیت"}</span>
        </button>
        {onToggleAppearance && <div className="appearance-switch" role="group" aria-label="ظاهر برنامه" dir="rtl">
          {([ ["light", "روشن", "sun"], ["dark", "تیره", "moon"] ] as const).map(([id, label, icon]) => (
            <button key={id} className="text-button appearance-button" type="button"
              aria-label={`استفاده از ظاهر ${label}`} aria-pressed={appearance === id}
              title={`ظاهر ${label}`} onClick={() => { if (appearance !== id) onToggleAppearance(); }}>
              {appearance === id && <motion.span className="selected-appearance" layoutId="selected-appearance" transition={transitions.layout} />}
              <span className="appearance-label"><InterfaceIcon name={icon} /><span>{label}</span></span>
            </button>
          ))}
        </div>}
        <button
          className="text-button tray-button"
          type="button"
          title="پنجره بسته می‌شود و برنامه در System tray فعال می‌ماند"
          aria-label="Minimize to system tray"
          onClick={onHideToTray}
        >
          <InterfaceIcon name="tray" /><span>Minimize</span>
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
        <h2 className="section-label">آپدیت DisRoute</h2>
        {phase === "available" && <><strong>نسخهٔ <bdi dir="ltr">{info?.version}</bdi> آماده است</strong><p>فایل نصب دانلود می‌شود؛ پس از بررسی امضا، اتصال Discord قطع و نصب آغاز می‌شود. کانفیگ ذخیره‌شده باقی می‌ماند.</p>{info?.notes && <details className="update-release-details" open><summary>تغییرات این آپدیت</summary><ReleasePreview notes={info.notes} /></details>}</>}
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

export function WorkspaceHeading({ view = "connection" }: { view?: AppView }) {
  return (
    <div className="workspace-heading">
      <div>
        <span className="workspace-eyebrow" dir="ltr">{view === "connection" ? "CONNECTION" : view === "guide" ? "GET STARTED" : "RELEASE NOTES"}</span>
        <h2>{view === "connection" ? "کنترل اتصال" : view === "guide" ? "شروع کار با DisRoute" : "تغییرات برنامه"}</h2>
        <p>{view === "connection" ? "Discord از مسیر پروکسی عبور می‌کند؛ بقیهٔ برنامه‌ها مستقیم می‌مانند." : view === "guide" ? "از آماده‌سازی تا اولین اتصال، قدم‌به‌قدم." : "امکانات و اصلاح‌های هر نسخه را اینجا ببینید."}</p>
      </div>
      <span className="local-badge"><i aria-hidden="true" /> فقط Discord</span>
    </div>
  );
}

export function ViewSwitch({ view, onChange }: { view: AppView; onChange: (view: AppView) => void }) {
  return (
    <nav className="view-switch" aria-label="بخش‌های برنامه">
      {([ ["connection", "اتصال"], ["guide", "راهنمای شروع"], ["whatsNew", "تازه‌های نسخه"] ] as const).map(([id, label]) => (
        <button type="button" key={id} aria-pressed={view === id} onClick={() => onChange(id)}>
          {view === id && <motion.span className="selected-view" layoutId="selected-view" transition={transitions.layout} />}
          <span className="navigation-label"><InterfaceIcon name={id} /><span>{label}</span></span>
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
