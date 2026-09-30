import type { PrerequisiteStatus } from "../types";
import { SpinnerIcon } from "./Icons";

interface Props {
  status: PrerequisiteStatus | null;
  busy: "" | "check" | "install";
  ready: boolean;
  connectionBusy: boolean;
  error: string;
  notice: string;
  onCheck: () => void;
  onInstall: () => void;
}

export function PrerequisitePanel({ status, busy, ready, connectionBusy, error, notice, onCheck, onInstall }: Props) {
  if (!status && !busy && !error) return null;
  const needsNetwork = status && (!status.packetFilter || !status.visualCpp);
  const canInstall = status?.supportedPlatform && status.elevated && status.dotNet && status.installerAvailable && needsNetwork;
  const checks = status ? [
    ["Windows Packet Filter", status.packetFilter], ["Visual C++ x64", status.visualCpp],
    [".NET Framework", status.dotNet], ["موتور شبکه", status.engines], ["دسترسی Administrator", status.elevated],
  ] as const : [];
  return (
    <section className={`setup-panel ${ready && status ? "setup-ready" : ""}`} aria-labelledby="setup-title" aria-busy={Boolean(busy)}>
      <div className="setup-heading">
        <div><h2 id="setup-title">{status && ready ? "سیستم آمادهٔ اتصال است" : "آماده‌سازی اتصال"}</h2>
          {(!ready || !status) && <p>پیش‌نیازهای اتصال اینجا بررسی و نصب می‌شوند.</p>}</div>
        <button className="text-button" type="button" onClick={onCheck} disabled={Boolean(busy)}>
          {busy === "check" && <SpinnerIcon />}{busy === "check" ? "در حال بررسی…" : "بررسی دوباره"}
        </button>
      </div>
      {status && !ready && <>
        <ul className="setup-checks">{checks.map(([name, installed]) => (
          <li key={name}><span dir="auto">{name}</span><span className={installed ? "setup-ok" : "setup-missing"}>{installed ? "آماده" : "نیاز به آماده‌سازی"}</span></li>
        ))}</ul>
        {!status.supportedPlatform && <p>این نسخه برای Windows x64 ساخته شده است.</p>}
        {!status.dotNet && <p><bdi dir="ltr">.NET Framework 4.7.2</bdi> یا جدیدتر لازم است. آن را از Windows Update نصب کنید و دوباره بررسی کنید.</p>}
        {!status.engines && <p>فایل‌های موتور پیدا نشدند. آخرین فایل نصب DisRoute را دوباره نصب کنید.</p>}
        {!status.elevated && <p>برنامه را ببندید و نسخهٔ نصب‌شده را دوباره باز کنید؛ اجازهٔ Administrator را در پیام Windows تأیید کنید.</p>}
        {needsNetwork && <div className="setup-install">
          <p>«نصب پیش‌نیازها» نصب‌کنندهٔ رسمی ProxiFyre را باز می‌کند. دانلود موارد لازم به اینترنت نیاز دارد؛ شرایط نصب در همان پنجره نمایش داده می‌شوند.</p>
          {!status.installerAvailable && <p>نصب‌کننده همراه این نسخه نیست؛ آخرین فایل نصب DisRoute را دریافت کنید.</p>}
          <button className="button button-secondary compact" type="button" onClick={onInstall} disabled={!canInstall || Boolean(busy) || connectionBusy}>
            {busy === "install" && <SpinnerIcon />}{busy === "install" ? "در انتظار پایان نصب…" : "نصب پیش‌نیازها"}
          </button>
          {connectionBusy && <p>پیش از نصب، اتصال را قطع کنید و منتظر پایان آزمایش یا آپدیت در حال اجرا بمانید.</p>}
        </div>}
      </>}
      {notice && <p role="status" className="setup-notice">{notice}</p>}
      {error && <p role="alert" className="setup-error">{error}</p>}
    </section>
  );
}
