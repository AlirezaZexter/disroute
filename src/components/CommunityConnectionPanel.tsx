import type { FormEvent } from "react";
import { AnimatePresence, motion } from "motion/react";
import type { CommunitySnapshot, CommunitySource, HealthResult } from "../types";
import { listItemVariants, transitions } from "../motion";
import { SpinnerIcon } from "./Icons";

export type CommunityBusy = "" | "refresh" | "scan" | "connect";

export interface SourceDraft {
  name: string;
  location: string;
  attribution: string;
  kind: CommunitySource["kind"];
  refreshIntervalMinutes: number;
  timeoutSeconds: number;
  expectedSha256: string;
  redistributionAuthorized: boolean;
}

const labelFa: Record<HealthResult["label"], string> = {
  Working: "فعال",
  Fast: "سریع",
  Unstable: "ناپایدار",
  "UDP unavailable": "بدون UDP",
  Untested: "آزمایش‌نشده",
  Offline: "خارج از دسترس",
};

function formatTime(value?: number | null) {
  return value
    ? new Intl.DateTimeFormat("fa-IR", { dateStyle: "short", timeStyle: "short" }).format(new Date(value * 1000))
    : "هنوز نوسازی نشده";
}

function CommunityProgress({ busy, hasResults, connected }: { busy: CommunityBusy; hasResults: boolean; connected: boolean }) {
  const steps = [
    { id: "refresh", label: "دریافت منابع" },
    { id: "scan", label: "آزمایش واقعی" },
    { id: "connect", label: "اتصال" },
  ] as const;
  const current = busy ? steps.findIndex((step) => step.id === busy) : connected ? 3 : hasResults ? 2 : -1;
  if (current < 0) return null;

  return (
    <motion.ol className="community-progress" initial={{ opacity: 0, y: 4 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} transition={transitions.normal} aria-label="مراحل اتصال سریع">
      {steps.map((step, index) => {
        const active = busy === step.id;
        const complete = index < current || connected;
        return (
          <li key={step.id} className={active ? "is-active" : complete ? "is-complete" : ""} aria-current={active ? "step" : undefined}>
            <span>{complete ? "✓" : index + 1}</span>
            <small>{step.label}</small>
          </li>
        );
      })}
    </motion.ol>
  );
}

interface CommunityConnectionPanelProps {
  community: CommunitySnapshot;
  results: HealthResult[];
  busy: CommunityBusy;
  error: string;
  warningChecked: boolean;
  sourceDraft: SourceDraft;
  connected: boolean;
  onSubmit: (event: FormEvent) => void;
  onWarningChecked: (value: boolean) => void;
  onAcknowledge: () => void;
  onRefresh: () => void;
  onScan: () => void;
  onCancelScan: () => void;
  onFailoverChange: (value: boolean) => void;
  onReplaceSources: (sources: CommunitySource[]) => void;
  onSourceDraftChange: (draft: SourceDraft) => void;
  onAddSource: () => void;
  onClearData: () => void;
}

export function CommunityConnectionPanel({
  community,
  results,
  busy,
  error,
  warningChecked,
  sourceDraft,
  connected,
  onSubmit,
  onWarningChecked,
  onAcknowledge,
  onRefresh,
  onScan,
  onCancelScan,
  onFailoverChange,
  onReplaceSources,
  onSourceDraftChange,
  onAddSource,
  onClearData,
}: CommunityConnectionPanelProps) {
  const best = results[0];
  const workingResults = results.filter((result) => result.working).slice(0, 5);
  const hasEnabledSource = community.sources.some((source) => source.enabled);

  return (
    <motion.form id="community-form" className="panel community-panel" noValidate onSubmit={onSubmit} layout>
      <div className="panel-heading">
        <div><h2>اتصال سریع رایگان</h2><p>منابع مجاز دریافت و با اتصال واقعی آزمایش می‌شوند.</p></div>
        <AnimatePresence initial={false}>
          {community.stale && <motion.span className="stale-badge" initial={{ opacity: 0, y: -3 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} transition={transitions.micro}>فهرست قدیمی</motion.span>}
        </AnimatePresence>
      </div>

      {!community.acknowledgedWarning ? (
        <motion.div className="community-warning" role="alertdialog" aria-labelledby="community-warning-title" initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={transitions.normal}>
          <strong id="community-warning-title">پیش از استفاده بخوانید</strong>
          <p>اتصال‌های رایگان توسط اشخاص ثالث ارائه می‌شوند. DisRoute مالک یا مدیر این سرورها نیست و امنیت، پایداری یا حریم خصوصی آن‌ها را تضمین نمی‌کند.</p>
          <label className="remember-option">
            <input type="checkbox" checked={warningChecked} onChange={(event) => onWarningChecked(event.target.checked)} />
            <span>این هشدار را خواندم و می‌پذیرم.</span>
          </label>
          <button className="button button-primary" type="button" disabled={!warningChecked} onClick={onAcknowledge}>ادامه</button>
        </motion.div>
      ) : (
        <motion.div className="community-content" initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={transitions.normal}>
          <div className="community-toolbar">
            <div><span>آخرین نوسازی</span><strong>{formatTime(community.refreshedAt)}</strong></div>
            <button className="button button-secondary compact" type="button" disabled={busy !== "" || !hasEnabledSource} aria-busy={busy === "refresh"} onClick={onRefresh}>
              {busy === "refresh" && <SpinnerIcon />}<span>{busy === "refresh" ? "در حال دریافت…" : "نوسازی منابع"}</span>
            </button>
            <button className="button button-primary compact" type="button" disabled={busy !== "" || community.candidates.length === 0} aria-busy={busy === "scan"} onClick={onScan}>
              {busy === "scan" && <SpinnerIcon />}<span>{busy === "scan" ? "در حال آزمایش…" : "آزمایش اتصال‌ها"}</span>
            </button>
            {busy === "scan" && <button className="text-button" type="button" onClick={onCancelScan}>لغو</button>}
          </div>

          <AnimatePresence initial={false}>
            {(busy !== "" || results.length > 0 || connected) && <CommunityProgress busy={busy} hasResults={results.length > 0} connected={connected} />}
          </AnimatePresence>

          <label className="remember-option">
            <input type="checkbox" checked={community.automaticFailover} onChange={(event) => onFailoverChange(event.target.checked)} />
            <span>تلاش خودکار با گزینهٔ سالم بعدی<small>در صورت شکست اتصال، حداکثر پنج گزینهٔ آزمایش‌شده بررسی می‌شوند.</small></span>
          </label>

          <AnimatePresence initial={false} mode="popLayout">
            {results.length > 0 && (
              <motion.section className="candidate-results" key="results" initial={{ opacity: 0, y: 6 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} transition={transitions.normal} aria-live="polite">
                <div className="best-candidate">
                  <span className="section-label">بهترین گزینهٔ فعلی</span>
                  {best.working ? (
                    <>
                      <div className="candidate-title"><strong>{best.sourceName}</strong><span>{labelFa[best.label]}</span></div>
                      <dl>
                        <div><dt>زمان اتصال</dt><dd><bdi dir="ltr">{best.medianLatencyMs ?? "N/A"} ms</bdi></dd></div>
                        <div><dt>پایداری</dt><dd>{Math.round((1 - best.failureRate) * 100).toLocaleString("fa-IR")}٪</dd></div>
                        <div><dt>وویس Discord</dt><dd>{best.udpAvailable ? "UDP فعال" : "تأیید نشده"}</dd></div>
                      </dl>
                      <small>منبع: {best.attribution}</small>
                    </>
                  ) : <p>در این آزمایش اتصال فعالی پیدا نشد.</p>}
                </div>
                {workingResults.length > 1 && (
                  <div className="candidate-list-wrap">
                    <span className="section-label">گزینه‌های آمادهٔ جایگزین</span>
                    <ol className="candidate-list">
                      {workingResults.map((result, index) => (
                        <motion.li layout key={result.candidateId} variants={listItemVariants} initial="initial" animate="enter">
                          <span>{index + 1}</span>
                          <div><strong>{result.sourceName}</strong><small>{result.country || result.protocol.toUpperCase()} · {labelFa[result.label]}</small></div>
                          <bdi dir="ltr">{result.medianLatencyMs ?? "N/A"} ms</bdi>
                        </motion.li>
                      ))}
                    </ol>
                  </div>
                )}
              </motion.section>
            )}
          </AnimatePresence>

          <details className="source-manager" open={community.sources.length === 0}>
            <summary>مدیریت منابع اتصال سریع</summary>
            <div className="source-manager-content">
              {community.sources.length === 0 && <p className="empty-state">هنوز منبعی اضافه نشده است. فقط منبعی را ثبت کنید که ارائه‌دهنده‌اش اجازهٔ بازنشر داده باشد.</p>}
              <motion.ul className="source-list" layout>
                <AnimatePresence initial={false}>
                  {community.sources.map((source) => (
                    <motion.li layout key={source.id} variants={listItemVariants} initial="initial" animate="enter" exit="exit">
                      <label className="source-toggle">
                        <input type="checkbox" checked={source.enabled} onChange={(event) => onReplaceSources(community.sources.map((item) => item.id === source.id ? { ...item, enabled: event.target.checked } : item))} />
                        <span><strong>{source.name}</strong><small>{source.attribution} · {formatTime(source.lastSuccessfulRefresh)}</small>{source.lastError && <em role="alert">{source.lastError}</em>}</span>
                      </label>
                      <button className="text-button danger-text" type="button" onClick={() => onReplaceSources(community.sources.filter((item) => item.id !== source.id))}>حذف</button>
                    </motion.li>
                  ))}
                </AnimatePresence>
              </motion.ul>

              <div className="source-form">
                <label><span>نوع منبع</span><select value={sourceDraft.kind} onChange={(event) => onSourceDraftChange({ ...sourceDraft, kind: event.target.value as CommunitySource["kind"] })}><option value="url">JSON manifest</option><option value="githubRaw">GitHub Raw</option><option value="githubRelease">GitHub release asset</option><option value="subscription">Subscription URL</option><option value="localFile">فایل محلی</option></select></label>
                <label><span>نام منبع</span><input required value={sourceDraft.name} onChange={(event) => onSourceDraftChange({ ...sourceDraft, name: event.target.value })} /></label>
                <label><span>نشانی یا مسیر</span><input required dir="ltr" value={sourceDraft.location} onChange={(event) => onSourceDraftChange({ ...sourceDraft, location: event.target.value.trim() })} /></label>
                <label><span>نام ارائه‌دهنده</span><input required value={sourceDraft.attribution} onChange={(event) => onSourceDraftChange({ ...sourceDraft, attribution: event.target.value })} /></label>
                <label><span>فاصلهٔ نوسازی (دقیقه)</span><input type="number" min={5} max={10080} value={sourceDraft.refreshIntervalMinutes} onChange={(event) => onSourceDraftChange({ ...sourceDraft, refreshIntervalMinutes: Number(event.target.value) })} /></label>
                <label><span>مهلت دریافت (ثانیه)</span><input type="number" min={5} max={120} value={sourceDraft.timeoutSeconds} onChange={(event) => onSourceDraftChange({ ...sourceDraft, timeoutSeconds: Number(event.target.value) })} /></label>
                <label className="full-field"><span><bdi dir="ltr">SHA-256</bdi> فایل (اختیاری)</span><input dir="ltr" maxLength={64} value={sourceDraft.expectedSha256} onChange={(event) => onSourceDraftChange({ ...sourceDraft, expectedSha256: event.target.value.trim() })} /></label>
                <label className="remember-option full-field"><input type="checkbox" checked={sourceDraft.redistributionAuthorized} onChange={(event) => onSourceDraftChange({ ...sourceDraft, redistributionAuthorized: event.target.checked })} /><span>ارائه‌دهنده اجازهٔ بازنشر این فهرست را داده است.</span></label>
                <button className="button button-secondary" type="button" disabled={!sourceDraft.name || !sourceDraft.location || !sourceDraft.attribution || !sourceDraft.redistributionAuthorized} onClick={onAddSource}>افزودن منبع مجاز</button>
              </div>
            </div>
          </details>

          <div className="community-footer">
            <p role={error ? "alert" : "status"} className={error ? "community-error is-error" : "community-error"}>{error || `${community.candidates.length.toLocaleString("fa-IR")} کانفیگ در کش موجود است.`}</p>
            <button className="text-button danger-text" type="button" onClick={onClearData}>پاک‌کردن کش و سابقه</button>
          </div>
        </motion.div>
      )}
    </motion.form>
  );
}
