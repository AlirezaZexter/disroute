import { useEffect, useState } from "react";
import type { ConnectionHealth } from "../types";
import { SpinnerIcon } from "./Icons";

const number = new Intl.NumberFormat("fa-IR");
const titles = {
  idle: "وضعیت زندهٔ اتصال",
  checking: "در انتظار پاسخ Discord",
  healthy: "Discord پاسخ می‌دهد",
  degraded: "پاسخ Discord دریافت نشد",
  unavailable: "مسیر اتصال پاسخ نمی‌دهد",
  recovering: "در حال بررسی اتصال جایگزین",
};

export function ConnectionHealthPanel({ health, connected, busy, error, onCheck }: {
  health: ConnectionHealth | null; connected: boolean; busy: boolean; error: string; onCheck: () => void;
}) {
  const [now, setNow] = useState(() => Date.now() / 1000);
  useEffect(() => {
    if (!health?.connectedAt) return;
    const timer = window.setInterval(() => setNow(Date.now() / 1000), 1000);
    return () => window.clearInterval(timer);
  }, [health?.connectedAt]);
  const stale = Boolean(health?.checkedAt && now - health.checkedAt > 45);
  const phase = stale && health?.phase === "healthy" ? "checking" : health?.phase ?? "idle";
  const samples = health?.samples ?? [];
  const elapsed = health?.connectedAt ? Math.max(0, Math.floor(now - health.connectedAt)) : 0;
  const duration = health?.connectedAt ? `${number.format(Math.floor(elapsed / 60))}:${String(elapsed % 60).padStart(2, "0").replace(/\d/g, d => "۰۱۲۳۴۵۶۷۸۹"[Number(d)])}` : "—";
  const lastCheck = health?.checkedAt ? new Date(health.checkedAt * 1000).toLocaleTimeString("fa-IR", { hour: "2-digit", minute: "2-digit", second: "2-digit" }) : null;
  const successful = samples.filter(s => s.latencyMs !== null).length;
  const note = phase === "idle" ? "بعد از اتصال، پاسخ Discord از همین مسیر بررسی می‌شود."
    : phase === "recovering" ? "گزینهٔ بعدی پیش از تعویض آزمایش می‌شود؛ ممکن است تماس نیاز به اتصال دوباره داشته باشد."
    : phase === "unavailable" ? "اگر اتصال برنگشت، اینترنت و کانفیگ را بررسی کنید."
    : phase === "degraded" ? "ممکن است موقت باشد؛ بررسی بعدی وضعیت را روشن می‌کند."
    : stale ? "بررسی قبلی قدیمی شده؛ منتظر نتیجهٔ تازه باشید."
    : "زمان پاسخ HTTPS است؛ کیفیت وویس و استریم را تضمین نمی‌کند.";
  return (
    <section className={`connection-health health-${phase}`} aria-label="وضعیت زندهٔ اتصال">
      <div className="health-heading">
        <div className="health-heading-copy">
          <span className="health-indicator" aria-hidden="true" />
          <h3 role="status" aria-live="polite" aria-atomic="true">{titles[phase]}</h3>
          {health?.mode && <span className="health-mode">{health.mode === "community" ? "اتصال رایگان" : "کانفیگ شخصی"}</span>}
        </div>
        <button type="button" className="text-button health-check" onClick={onCheck} disabled={!connected || busy}>
          {busy && <SpinnerIcon />}{busy ? "در حال بررسی…" : "بررسی همین حالا"}
        </button>
      </div>
      <div className="health-body">
        <div className="health-history">
          <div className="health-history-heading"><span>بررسی‌های اخیر</span><span>{lastCheck ? `آخرین بررسی ${lastCheck}` : "هنوز اندازه‌گیری نشده"}</span></div>
          <div className="health-bars" dir="ltr" role="img" aria-label={samples.length ? `${number.format(successful)} بررسی موفق از ${number.format(samples.length)} بررسی اخیر` : "هنوز نتیجه‌ای ثبت نشده است"}>
            {Array.from({ length: 24 }, (_, i) => {
              const sample = samples[i - (24 - samples.length)];
              const failed = sample && sample.latencyMs === null;
              return <span key={i} className={!sample ? "empty" : failed ? "failed" : "passed"}
                style={{ height: !sample ? "18%" : failed ? "100%" : `${Math.max(22, Math.min(100, (sample.latencyMs ?? 0) / 20))}%` }}
                title={sample ? failed ? "بدون پاسخ" : `${number.format(sample.latencyMs!)} میلی‌ثانیه` : undefined}>{failed ? "×" : ""}</span>;
            })}
          </div>
          <div className="health-history-legend"><span>قدیمی‌تر</span><span>جدیدتر</span></div>
        </div>
        <dl className="health-metrics">
          <div><dt>زمان پاسخ</dt><dd>{!stale && health?.latencyMs != null ? <>{number.format(health.latencyMs)} <small>ms</small></> : "—"}</dd></div>
          <div><dt>مدت جلسه</dt><dd dir="ltr">{duration}</dd></div>
          <div><dt>بررسی موفق</dt><dd>{samples.length ? `${number.format(successful)} از ${number.format(samples.length)}` : "—"}</dd></div>
        </dl>
      </div>
      <div className="health-footnote"><p>{error || note}</p>{Boolean(health?.recoveries) && <span>{number.format(health!.recoveries)} تعویض اتصال</span>}</div>
    </section>
  );
}
