import { FormEvent, useEffect, useRef, useState } from "react";
import { cancelCommunityScan, clearCommunityData, connect, connectCommunity, disconnect, getCommunitySnapshot, getStatus, saveCommunitySources, saveProfile, loadProfile, forgetProfile, hideToTray, refreshCommunity, restartDiscord, scanCommunity, setCommunityPreferences } from "./api";
import { AnimatePresence, LayoutGroup, MotionConfig, motion, useReducedMotion, useIsPresent } from "motion/react";
import type { AppStatus, CommunitySnapshot, CommunitySource, HealthResult, ProxyProfile } from "./types";
import { downloadAndInstall, findUpdate, type UpdateProgress } from "./updater";
import type { Update } from "@tauri-apps/plugin-updater";

const initialProfile: ProxyProfile = {
  name: "Discord",
  configLink: "",
};

const protocolNames = {
  vless: "VLESS",
  vmess: "VMess",
  trojan: "Trojan",
  ss: "Shadowsocks",
} as const;

function detectProtocol(value: string) {
  const scheme = value.trim().match(/^([a-z][a-z0-9+.-]*):\/\//i)?.[1]?.toLowerCase();
  return scheme && scheme in protocolNames ? protocolNames[scheme as keyof typeof protocolNames] : null;
}

const emptyStatus: AppStatus = {
  status: "disconnected",
  engineReady: false,
  isElevated: false,
  message: "در حال بررسی موتور…",
};

const emptyCommunity: CommunitySnapshot = { sources: [], candidates: [], stale: false, acknowledgedWarning: false, automaticFailover: true };
const labelFa: Record<HealthResult["label"], string> = { Working: "فعال", Fast: "سریع", Unstable: "ناپایدار", "UDP unavailable": "بدون UDP", Untested: "آزمایش‌نشده", Offline: "خارج از دسترس" };

function sourceId() { return `source-${Date.now().toString(36)}`; }
function formatTime(value?: number | null) { return value ? new Intl.DateTimeFormat("fa-IR", { dateStyle: "short", timeStyle: "short" }).format(new Date(value * 1000)) : "هنوز نوسازی نشده"; }

function ShieldIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 2 4.5 5v5.7c0 4.8 3 9.2 7.5 11.3 4.5-2.1 7.5-6.5 7.5-11.3V5L12 2Z" />
      <path d="m8.8 12 2 2 4.4-4.5" />
    </svg>
  );
}

function BidirectionalRouteIcon() {
  return (
    <svg viewBox="0 0 28 12" aria-hidden="true" focusable="false">
      <path d="M2 6h24M6 2 2 6l4 4M22 2l4 4-4 4" />
    </svg>
  );
}

function StatusHeading({ title, reduced }: { title: string; reduced: boolean }) {
  const present = useIsPresent();
  return <motion.h2 aria-hidden={!present} initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: reduced ? 0 : .18 }}>{title}</motion.h2>;
}

function App() {
  const [profile, setProfile] = useState(initialProfile);
  const [appStatus, setAppStatus] = useState(emptyStatus);
  const [busy, setBusy] = useState(false);
  const [discordBusy, setDiscordBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const [remember, setRemember] = useState(true);
  const [saved, setSaved] = useState(false);
  const [notice, setNotice] = useState("");
  const [showSecret, setShowSecret] = useState(false);
  const [configTouched, setConfigTouched] = useState(false);
  const [confirmForget, setConfirmForget] = useState(false);
  const [view, setView] = useState<"connection" | "guide">("connection");
  const [connectionMode, setConnectionMode] = useState<"personal" | "community">("personal");
  const [community, setCommunity] = useState<CommunitySnapshot>(emptyCommunity);
  const [communityResults, setCommunityResults] = useState<HealthResult[]>([]);
  const [communityBusy, setCommunityBusy] = useState<"" | "refresh" | "scan" | "connect">("");
  const [communityError, setCommunityError] = useState("");
  const [warningChecked, setWarningChecked] = useState(false);
  const [sourceDraft, setSourceDraft] = useState({ name: "", location: "", attribution: "", kind: "url" as CommunitySource["kind"], refreshIntervalMinutes: 60, timeoutSeconds: 12, expectedSha256: "", redistributionAuthorized: false });
  const [updatePhase, setUpdatePhase] = useState<"idle" | "checking" | "available" | "current" | "downloading" | "installing" | "error">("idle");
  const [updateInfo, setUpdateInfo] = useState<{ version: string; notes?: string } | null>(null);
  const [updateProgress, setUpdateProgress] = useState<UpdateProgress>({ downloaded: 0 });
  const [updateError, setUpdateError] = useState("");
  const pendingUpdate = useRef<Update | null>(null);
  const reduced = useReducedMotion();
  const connected = appStatus.status === "connected";
  const detectedProtocol = detectProtocol(profile.configLink);
  const hasConfig = profile.configLink.trim().length > 0;
  const unsupportedConfig = hasConfig && !detectedProtocol;
  const configInvalid = configTouched && unsupportedConfig;
  const statusTitle = {
    disconnected: "آمادهٔ اتصال",
    connecting: "در حال اتصال",
    connected: "متصل است",
    error: "خطای اتصال",
  }[appStatus.status];

  useEffect(() => {
    getStatus().then(setAppStatus).catch((error: unknown) => {
      setAppStatus({ ...emptyStatus, status: "error", message: String(error) });
    });
    loadProfile().then((value) => {
      if (value) { setProfile(value); setSaved(true); setNotice("پروفایل ذخیره‌شده بازیابی شد."); }
    }).catch(() => setNotice("بازیابی پروفایل ممکن نشد؛ کانفیگ را دوباره وارد و ذخیره کنید."))
      .finally(() => setLoading(false));
    getCommunitySnapshot().then(setCommunity).catch(() => setCommunityError("خواندن تنظیمات اتصال سریع ممکن نشد."));
  }, []);

  useEffect(() => {
    if (busy || discordBusy || !connected) return;
    const timer = window.setInterval(() => { getStatus().then(setAppStatus).catch(() => {}); }, 5000);
    return () => window.clearInterval(timer);
  }, [busy, discordBusy, connected]);

  const update = <K extends keyof ProxyProfile>(key: K, value: ProxyProfile[K]) =>
    { setProfile((current) => ({ ...current, [key]: value })); setNotice("تغییرات هنوز ذخیره نشده‌اند."); };

  async function persist() {
    if (remember) { await saveProfile(profile); setSaved(true); setNotice("کانفیگ ذخیره شد."); }
    else { await forgetProfile(); setSaved(false); setNotice("کانفیگ ذخیره نمی‌شود."); }
  }

  async function handleSave() {
    setBusy(true);
    try { await persist(); } catch (error) { setNotice(String(error)); }
    finally { setBusy(false); }
  }

  async function handleForget() {
    setBusy(true);
    try { await forgetProfile(); setSaved(false); setRemember(false); setConfirmForget(false); setProfile(initialProfile); setNotice("کانفیگ ذخیره‌شده حذف شد."); }
    catch (error) { setNotice(String(error)); }
    finally { setBusy(false); }
  }

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setAppStatus((current) => ({ ...current, status: "connecting", message: "در حال اتصال…" }));
    try {
      await persist();
      setAppStatus(await connect(profile));
    } catch (error) {
      setAppStatus((current) => ({ ...current, status: "error", message: String(error) }));
    } finally {
      setBusy(false);
    }
  }

  async function handleDisconnect() {
    setBusy(true);
    try {
      setAppStatus(await disconnect());
    } catch (error) {
      setAppStatus((current) => ({ ...current, status: "error", message: String(error) }));
    } finally {
      setBusy(false);
    }
  }

  async function acknowledgeCommunity() {
    if (!warningChecked) return;
    try { setCommunity(await setCommunityPreferences(true, community.automaticFailover)); }
    catch (error) { setCommunityError(String(error)); }
  }

  async function updateFailover(enabled: boolean) {
    try { setCommunity(await setCommunityPreferences(community.acknowledgedWarning, enabled)); }
    catch (error) { setCommunityError(String(error)); }
  }

  async function addSource() {
    const source: CommunitySource = { id: sourceId(), ...sourceDraft, expectedSha256: sourceDraft.expectedSha256 || null, enabled: true };
    try {
      setCommunity(await saveCommunitySources([...community.sources, source]));
      setSourceDraft({ name: "", location: "", attribution: "", kind: "url", refreshIntervalMinutes: 60, timeoutSeconds: 12, expectedSha256: "", redistributionAuthorized: false });
      setCommunityError("");
    } catch (error) { setCommunityError(String(error)); }
  }

  async function replaceSources(sources: CommunitySource[]) {
    try { setCommunity(await saveCommunitySources(sources)); setCommunityError(""); }
    catch (error) { setCommunityError(String(error)); }
  }

  async function handleCommunityRefresh() {
    setCommunityBusy("refresh"); setCommunityError(""); setCommunityResults([]);
    try { setCommunity(await refreshCommunity()); }
    catch (error) { setCommunityError(String(error)); }
    finally { setCommunityBusy(""); }
  }

  async function handleCommunityScan() {
    setCommunityBusy("scan"); setCommunityError(""); setCommunityResults([]);
    try { setCommunityResults(await scanCommunity()); }
    catch (error) { setCommunityError(String(error)); }
    finally { setCommunityBusy(""); }
  }

  async function handleCommunityConnect(event: FormEvent) {
    event.preventDefault();
    if (!community.acknowledgedWarning || communityBusy) return;
    setCommunityError("");
    try {
      setCommunityBusy("refresh");
      const snapshot = await refreshCommunity();
      setCommunity(snapshot);
      if (!snapshot.candidates.length) throw new Error("منابع فعلی کانفیگ قابل‌آزمایشی ندارند. وضعیت منابع را بررسی کنید.");
      setCommunityBusy("scan");
      const results = await scanCommunity();
      setCommunityResults(results);
      const healthy = results.filter((result) => result.working).map((result) => result.candidateId);
      if (!healthy.length) throw new Error("در این آزمایش هیچ اتصال فعالی به Discord پیدا نشد. بعداً دوباره امتحان کنید.");
      setCommunityBusy("connect");
      setAppStatus(await connectCommunity(healthy));
    }
    catch (error) { setCommunityError(String(error)); }
    finally { setCommunityBusy(""); }
  }

  async function handleRestartDiscord() {
    setDiscordBusy(true);
    setAppStatus((current) => ({ ...current, message: "در حال راه‌اندازی مجدد Discord…" }));
    try {
      const message = await restartDiscord();
      setAppStatus((current) => ({ ...current, message }));
    } catch (error) {
      setAppStatus((current) => ({
        ...current,
        message: `اتصال روشن است؛ راه‌اندازی مجدد Discord ناموفق بود: ${String(error)}`,
      }));
    } finally {
      setDiscordBusy(false);
    }
  }

  async function handleCheckUpdates() {
    setUpdatePhase("checking");
    setUpdateError("");
    try {
      const update = await findUpdate();
      pendingUpdate.current = update;
      if (update) {
        setUpdateInfo({ version: update.version, notes: update.body || undefined });
        setUpdatePhase("available");
      } else {
        setUpdateInfo(null);
        setUpdatePhase("current");
      }
    } catch (error) {
      setUpdateError(`بررسی نسخه ناموفق بود: ${String(error)}`);
      setUpdatePhase("error");
    }
  }

  async function handleInstallUpdate() {
    const update = pendingUpdate.current;
    if (!update) return;
    setUpdateProgress({ downloaded: 0 });
    setUpdateError("");
    setUpdatePhase("downloading");
    try {
      await downloadAndInstall(
        update,
        (progress) => setUpdateProgress(progress),
        async () => {
          setUpdatePhase("installing");
          if (connected) setAppStatus(await disconnect());
        },
      );
    } catch (error) {
      setUpdateError(`دانلود یا نصب نسخه جدید ناموفق بود: ${String(error)}`);
      setUpdatePhase("error");
    }
  }

  const checkingUpdate = updatePhase === "checking";
  const updating = updatePhase === "downloading" || updatePhase === "installing";

  return (
    <MotionConfig reducedMotion="user" transition={{ duration: reduced ? 0 : .22 }}><LayoutGroup>
    <main className="shell">
      <header className="topbar">
        <div className="brand-lockup">
          <div className="brand-mark"><ShieldIcon /></div>
          <div>
          <h1>DisRoute</h1>
            <p>مسیر مستقل Discord</p>
          </div>
        </div>
        <div className="header-actions">
          <span className="version" dir="ltr">v0.5.0</span>
          <button className="text-button update-check-button" type="button" disabled={checkingUpdate || updating} onClick={handleCheckUpdates}>{checkingUpdate ? "در حال بررسی…" : updating ? "در حال آپدیت…" : "بررسی آپدیت"}</button>
          <button className="text-button tray-button" type="button" title="پنجره بسته می‌شود و برنامه در System tray فعال می‌ماند" onClick={() => hideToTray().catch((error) => setNotice(String(error)))}>Minimize to system tray</button>
        </div>
      </header>

      <AnimatePresence initial={false}>
        {updatePhase !== "idle" && updatePhase !== "checking" && (
          <motion.section className={`update-card update-${updatePhase}`} initial={{ opacity: 0, y: reduced ? 0 : -6 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} role={updatePhase === "error" ? "alert" : "status"} aria-live={updatePhase === "error" ? "assertive" : "polite"}>
            <div className="update-copy">
              <span className="section-label">آپدیت DisRoute</span>
              {updatePhase === "available" && <><strong>نسخه <bdi dir="ltr">{updateInfo?.version}</bdi> آماده است</strong><p>{updateInfo?.notes || "نسخه جدید از GitHub دانلود و پس از بررسی امضا نصب می‌شود."}</p></>}
              {updatePhase === "current" && <><strong>نسخه جدیدی منتشر نشده</strong><p>همین نسخه، آخرین نسخه موجود است.</p></>}
              {updatePhase === "downloading" && <><strong>در حال دانلود نسخه <bdi dir="ltr">{updateInfo?.version}</bdi></strong><p>{updateProgress.percent === undefined ? "حجم فایل در حال دریافت است…" : `${updateProgress.percent}٪ دریافت شده`}</p></>}
              {updatePhase === "installing" && <><strong>دانلود کامل شد</strong><p>نصب شروع شده و DisRoute دوباره اجرا می‌شود.</p></>}
              {updatePhase === "error" && <><strong>آپدیت انجام نشد</strong><p dir="auto">{updateError}</p></>}
            </div>
            {updatePhase === "downloading" && <div className="update-progress" role="progressbar" aria-label="پیشرفت دانلود آپدیت" aria-valuemin={0} aria-valuemax={100} aria-valuenow={updateProgress.percent}><span style={{ transform: `scaleX(${(updateProgress.percent || 0) / 100})` }} /></div>}
            <div className="update-actions">
              {updatePhase === "available" && <button className="button button-primary" type="button" onClick={handleInstallUpdate}>دانلود و نصب</button>}
              {(updatePhase === "available" || updatePhase === "current" || updatePhase === "error") && <button className="text-button" type="button" onClick={() => setUpdatePhase("idle")}>{updatePhase === "available" ? "بعداً" : "بستن"}</button>}
            </div>
          </motion.section>
        )}
      </AnimatePresence>

      <div className="workspace-heading"><div><h2>کنترل اتصال</h2><p>Discord را از مسیر پروکسی عبور دهید؛ بقیهٔ برنامه‌ها مستقیم می‌مانند.</p></div><span className="local-badge"><i /> مسیریابی انتخابی</span></div>
      <nav className="view-switch" aria-label="بخش‌های برنامه">
        {([['connection', 'اتصال'], ['guide', 'راهنمای شروع']] as const).map(([id, label]) => <button type="button" key={id} aria-pressed={view === id} onClick={() => setView(id)}>{view === id && <motion.span className="selected-view" layoutId={reduced ? undefined : 'selected-view'} transition={{ type: 'spring', stiffness: 380, damping: 32 }} />}<span>{label}</span></button>)}
      </nav>

      <section
        className={`status-card status-${appStatus.status}`}
        role={appStatus.status === "error" ? "alert" : "status"}
        aria-live={appStatus.status === "error" ? "assertive" : "polite"}
      >
        <div className="status-orb"><span /></div>
        <div className="status-copy">
          <span className="section-label">وضعیت فعلی</span>
          <div className="status-title-stack"><AnimatePresence initial={false}><StatusHeading key={statusTitle} title={statusTitle} reduced={!!reduced} /></AnimatePresence></div>
          <p dir="auto">{appStatus.message}</p>
        </div>
        <div className="status-controls"><div className="actions">
            {connected ? (
              <><button className="button button-secondary" type="button" onClick={handleRestartDiscord} disabled={busy || discordBusy}>{discordBusy ? "در حال اجرا…" : "Restart Discord"}</button><button className="button button-danger" type="button" onClick={handleDisconnect} disabled={busy || discordBusy}>قطع اتصال</button></>
            ) : (
              <button className="button button-primary" type="submit" form={connectionMode === "personal" ? "connection-form" : "community-form"} disabled={connectionMode === "personal" ? loading || busy || !hasConfig || unsupportedConfig : communityBusy !== "" || !community.acknowledgedWarning || !community.sources.some((source) => source.enabled)}>{busy || communityBusy ? "لطفاً صبر کنید…" : "اتصال Discord"}</button>
            )}
          </div><div className="route-map" aria-label="مسیر دوطرفهٔ شبکه بین Discord، Proxy و Internet">
          <span>Discord</span><BidirectionalRouteIcon /><span>Proxy</span><BidirectionalRouteIcon /><span>Internet</span>
        </div></div>
      </section>

      <div className="content-grid" hidden={view !== 'connection'}>
        <div className="mode-column">
        <nav className="mode-switch" aria-label="روش اتصال">
          <button type="button" aria-pressed={connectionMode === "personal"} onClick={() => setConnectionMode("personal")}>کانفیگ شخصی</button>
          <button type="button" aria-pressed={connectionMode === "community"} onClick={() => setConnectionMode("community")}>اتصال سریع رایگان</button>
        </nav>
        {connectionMode === "personal" ? (
        <form id="connection-form" className="panel" onSubmit={handleSubmit}>
          <div className="panel-heading">
            <div><h2>کانفیگ شخصی</h2><p>لینک اتصال خودتان را وارد کنید.</p></div>
            <span className="protocol-pill" aria-live="polite">{detectedProtocol || "Xray"}</span>
          </div>

          <label>
            <span>نام پروفایل</span>
            <input required disabled={loading || busy || connected} value={profile.name} onChange={(e) => update("name", e.target.value)} autoComplete="off" />
          </label>
          <label>
            <span>لینک اتصال</span>
            <input id="config-link" disabled={loading || busy || connected} dir="ltr" type={showSecret ? "text" : "password"} required aria-describedby="config-help" aria-invalid={configInvalid} placeholder="vless:// · vmess:// · trojan:// · ss://" value={profile.configLink} onChange={(e) => update("configLink", e.target.value.trim())} onBlur={() => setConfigTouched(true)} autoComplete="off" spellCheck={false} />
            <small id="config-help" className={configInvalid ? "field-error" : undefined}>{configInvalid ? "این نوع لینک پشتیبانی نمی‌شود." : <><bdi dir="ltr">VLESS، VMess، Trojan و Shadowsocks</bdi> پشتیبانی می‌شوند.</>}</small>
          </label>
          <div className="profile-tools"><button className="text-button" type="button" aria-pressed={showSecret} onClick={() => setShowSecret(!showSecret)}>{showSecret ? 'پنهان کردن لینک' : 'نمایش لینک'}</button><span className="saved-badge">{loading ? 'در حال بارگذاری…' : saved ? 'ذخیره شده' : 'ذخیره نشده'}</span></div>
          <label className="remember-option"><input type="checkbox" checked={remember} disabled={busy || loading || connected} onChange={(e) => { setRemember(e.target.checked); setNotice('برای اعمال این انتخاب، ذخیره را بزنید.'); }} /><span>کانفیگ برای دفعات بعد ذخیره شود<small>رمزگذاری با حساب ویندوز شما؛ بدون ذخیره در مرورگر</small></span></label>

          <div className="profile-tools"><button className="text-button" type="button" disabled={loading || busy || connected || !hasConfig || unsupportedConfig} onClick={handleSave}>ذخیره تنظیمات</button>{saved && <button className="text-button danger-text" type="button" disabled={busy || connected} onClick={() => setConfirmForget(!confirmForget)}>حذف کانفیگ ذخیره‌شده</button>}</div>
          {confirmForget && <div className="delete-confirm"><p>کانفیگ ذخیره‌شده حذف شود؟ برای اتصال بعدی باید دوباره واردش کنید.</p><button className="text-button danger-text" type="button" disabled={busy} onClick={handleForget}>بله، حذف شود</button><button className="text-button" type="button" onClick={() => setConfirmForget(false)}>انصراف</button></div>}
          <p className="privacy-note" role="status">{notice || 'لینک کانفیگ را وارد کنید.'}</p>
        </form>
        ) : (
          <form id="community-form" className="panel community-panel" noValidate onSubmit={handleCommunityConnect}>
            <div className="panel-heading"><div><h2>اتصال سریع رایگان</h2><p>گزینه‌های منابع تأییدشده را همین‌جا آزمایش کنید.</p></div>{community.stale && <span className="stale-badge">فهرست قدیمی</span>}</div>
            {!community.acknowledgedWarning ? <div className="community-warning" role="alertdialog" aria-labelledby="community-warning-title">
              <strong id="community-warning-title">پیش از استفاده بخوانید</strong>
              <p>اتصال‌های رایگان توسط اشخاص ثالث ارائه می‌شوند. Disroute مالک یا مدیر این سرورها نیست و امنیت، پایداری یا حریم خصوصی آن‌ها را تضمین نمی‌کند.</p>
              <label className="remember-option"><input type="checkbox" checked={warningChecked} onChange={(event) => setWarningChecked(event.target.checked)} /><span>این هشدار را خواندم و می‌پذیرم.</span></label>
              <button className="button button-primary" type="button" disabled={!warningChecked} onClick={acknowledgeCommunity}>ادامه</button>
            </div> : <>
              <div className="community-toolbar"><div><span>آخرین نوسازی</span><strong>{formatTime(community.refreshedAt)}</strong></div><button className="button button-secondary compact" type="button" disabled={communityBusy !== "" || !community.sources.some((source) => source.enabled)} onClick={handleCommunityRefresh}>{communityBusy === "refresh" ? "در حال دریافت…" : "نوسازی منابع"}</button><button className="button button-primary compact" type="button" disabled={communityBusy !== "" || community.candidates.length === 0} onClick={handleCommunityScan}>{communityBusy === "scan" ? "در حال آزمایش…" : "آزمایش اتصال‌ها"}</button>{communityBusy === "scan" && <button className="text-button" type="button" onClick={() => cancelCommunityScan()}>لغو</button>}</div>
              <label className="remember-option"><input type="checkbox" checked={community.automaticFailover} onChange={(event) => updateFailover(event.target.checked)} /><span>تلاش خودکار با گزینهٔ سالم بعدی<small>در صورت شکست اتصال، حداکثر پنج گزینهٔ آزمایش‌شده بررسی می‌شوند.</small></span></label>
              {communityResults.length > 0 && <section className="best-candidate" aria-live="polite"><span className="section-label">بهترین گزینهٔ فعلی</span>{communityResults[0].working ? <><div className="candidate-title"><strong>{communityResults[0].sourceName}</strong><span>{labelFa[communityResults[0].label]}</span></div><dl><div><dt>زمان اتصال</dt><dd><bdi dir="ltr">{communityResults[0].medianLatencyMs} ms</bdi></dd></div><div><dt>پایداری</dt><dd>{Math.round((1 - communityResults[0].failureRate) * 100)}٪</dd></div><div><dt>وویس Discord</dt><dd>{communityResults[0].udpAvailable ? "UDP فعال" : "تأیید نشده"}</dd></div></dl><small>منبع: {communityResults[0].attribution}</small></> : <p>در این آزمایش اتصال فعالی پیدا نشد.</p>}</section>}
              <details className="source-manager" open={community.sources.length === 0}><summary>مدیریت منابع اتصال سریع</summary>
                {community.sources.length === 0 && <p className="empty-state">هیچ منبعی به‌صورت پیش‌فرض اضافه نشده است. فقط منبعی را ثبت کنید که ارائه‌دهنده‌اش اجازهٔ بازنشر داده باشد.</p>}
                <ul className="source-list">{community.sources.map((source) => <li key={source.id}><label className="source-toggle"><input type="checkbox" checked={source.enabled} onChange={(event) => replaceSources(community.sources.map((item) => item.id === source.id ? { ...item, enabled: event.target.checked } : item))} /><span><strong>{source.name}</strong><small>{source.attribution} · {formatTime(source.lastSuccessfulRefresh)}</small>{source.lastError && <em>{source.lastError}</em>}</span></label><button className="text-button danger-text" type="button" onClick={() => replaceSources(community.sources.filter((item) => item.id !== source.id))}>حذف</button></li>)}</ul>
                <div className="source-form"><label><span>نوع منبع</span><select value={sourceDraft.kind} onChange={(event) => setSourceDraft({ ...sourceDraft, kind: event.target.value as CommunitySource["kind"] })}><option value="url">JSON manifest</option><option value="githubRaw">GitHub Raw</option><option value="githubRelease">GitHub release asset</option><option value="subscription">Subscription URL</option><option value="localFile">فایل محلی</option></select></label><label><span>نام منبع</span><input required value={sourceDraft.name} onChange={(event) => setSourceDraft({ ...sourceDraft, name: event.target.value })} /></label><label><span>نشانی یا مسیر</span><input required dir="ltr" value={sourceDraft.location} onChange={(event) => setSourceDraft({ ...sourceDraft, location: event.target.value.trim() })} /></label><label><span>نام ارائه‌دهنده</span><input required value={sourceDraft.attribution} onChange={(event) => setSourceDraft({ ...sourceDraft, attribution: event.target.value })} /></label><label><span>فاصلهٔ نوسازی (دقیقه)</span><input type="number" min={5} max={10080} value={sourceDraft.refreshIntervalMinutes} onChange={(event) => setSourceDraft({ ...sourceDraft, refreshIntervalMinutes: Number(event.target.value) })} /></label><label><span>مهلت دریافت (ثانیه)</span><input type="number" min={5} max={120} value={sourceDraft.timeoutSeconds} onChange={(event) => setSourceDraft({ ...sourceDraft, timeoutSeconds: Number(event.target.value) })} /></label><label className="full-field"><span>SHA-256 فایل (اختیاری)</span><input dir="ltr" maxLength={64} value={sourceDraft.expectedSha256} onChange={(event) => setSourceDraft({ ...sourceDraft, expectedSha256: event.target.value.trim() })} /></label><label className="remember-option full-field"><input type="checkbox" checked={sourceDraft.redistributionAuthorized} onChange={(event) => setSourceDraft({ ...sourceDraft, redistributionAuthorized: event.target.checked })} /><span>ارائه‌دهنده اجازهٔ بازنشر این فهرست را داده است.</span></label><button className="button button-secondary" type="button" disabled={!sourceDraft.name || !sourceDraft.location || !sourceDraft.attribution || !sourceDraft.redistributionAuthorized} onClick={addSource}>افزودن منبع مجاز</button></div>
              </details>
              <div className="community-footer"><p role="status" className="community-error">{communityError || `${community.candidates.length.toLocaleString("fa-IR")} کانفیگ در کش موجود است.`}</p><button className="text-button danger-text" type="button" onClick={async () => { setCommunity(await clearCommunityData()); setCommunityResults([]); }}>پاک‌کردن کش و سابقه</button></div>
            </>}
          </form>
        )}
        </div>

        <aside className="panel side-panel">
          <div className="panel-heading"><div><h2>مسیر ترافیک</h2><p>فقط Discord از پروکسی عبور می‌کند.</p></div></div>
          <ul className="app-list">
            <li className="active"><span className="app-dot discord" /><div><strong>Discord</strong><small>TCP و UDP از پروکسی</small></div><span>Proxy</span></li>
            <li><span className="app-dot game" /><div><strong>بازی‌ها</strong><small>بدون تغییر مسیر</small></div><span>Direct</span></li>
            <li><span className="app-dot browser" /><div><strong>سایر برنامه‌ها</strong><small>اینترنت عادی</small></div><span>Direct</span></li>
          </ul>
          <div className="requirement">
            <strong>{appStatus.engineReady ? "موتور شبکه آماده است" : "فایل‌های موتور پیدا نشد"}</strong>
            <p>در اولین اتصال، برنامه مجوز Firewall موردنیاز را برای موتور شبکه اضافه می‌کند و اتصال پروکسی را بررسی می‌کند.</p>
            <p>{appStatus.isElevated ? 'دسترسی Administrator فعال است.' : 'برای اتصال، برنامه را با Run as administrator اجرا کنید.'}</p>
          </div>
        </aside>
      </div>
      <AnimatePresence>{view === 'guide' && <motion.section className="panel guide-panel" initial={{ opacity: 0, y: reduced ? 0 : 8 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }}>
        <div className="guide-intro"><div><span className="section-label">شروع کار</span><h2>راه‌اندازی DisRoute</h2><p>نصب اولیه فقط یک‌بار انجام می‌شود. بعد از آن، اتصال از داخل همین برنامه در دسترس است.</p></div><span className="guide-time">حدود ۳ دقیقه</span></div>
        <div className="guide-steps">
          <article><span className="step-number">۱</span><div><strong>پیش‌نیاز شبکه را نصب کنید</strong><p>بستهٔ <bdi dir="ltr">ProxiFyre</bdi> را از صفحهٔ انتشار بگیرید و فایل نصب <bdi dir="ltr">Windows Packet Filter</bdi> را یک‌بار اجرا کنید. موتورهای برنامه همراه نصب‌کنندهٔ DisRoute هستند.</p></div></article>
          <article><span className="step-number">۲</span><div><strong>DisRoute را با دسترسی مدیر اجرا کنید</strong><p>روی میان‌بُر برنامه راست‌کلیک کنید و <bdi dir="ltr">Run as administrator</bdi> را بزنید. این دسترسی برای قانون Firewall و مسیریابی پردازش Discord لازم است.</p></div></article>
          <article><span className="step-number">۳</span><div><strong>روش اتصال را انتخاب کنید</strong><p>در «کانفیگ شخصی» لینک <bdi dir="ltr">VLESS، VMess، Trojan یا Shadowsocks</bdi> را وارد کنید. اگر کانفیگ ندارید، «اتصال سریع رایگان» منابع فعال را آزمایش می‌کند؛ هشدار سرورهای شخص ثالث را پیش از استفاده بخوانید.</p></div></article>
          <article><span className="step-number">۴</span><div><strong>اتصال Discord را بزنید</strong><p>بعد از نمایش وضعیت «متصل است»، Discord را باز کنید. اگر از قبل باز بوده و متصل نشد، <bdi dir="ltr">Restart Discord</bdi> را بزنید.</p></div></article>
          <article><span className="step-number">۵</span><div><strong>تماس صوتی و استریم را جداگانه بررسی کنید</strong><p>پیام و تماس صوتی مسیر یکسانی ندارند. برای تماس صوتی و استریم، کانفیگ باید <bdi dir="ltr">UDP</bdi> و آپلود مناسب داشته باشد. کندی بازی هم می‌تواند از مصرف هم‌زمان پهنای باند باشد.</p></div></article>
          <article><span className="step-number">۶</span><div><strong>برنامه را به کنار ساعت بفرستید</strong><p><bdi dir="ltr">Minimize to system tray</bdi> فقط پنجره را می‌بندد و اتصال روشن می‌ماند. برای خروج کامل، از منوی آیکون DisRoute کنار ساعت ویندوز استفاده کنید.</p></div></article>
        </div>
        <div className="guide-note"><strong>آپدیت داخل برنامه</strong><p>از بالای صفحه «بررسی آپدیت» را بزنید. نسخهٔ جدید پس از دانلود و بررسی امضا نصب می‌شود؛ آپدیت خودکار و بی‌صدا انجام نمی‌شود.</p></div>
      </motion.section>}</AnimatePresence>
      <footer className="app-footer"><span>بستن پنجره: <bdi dir="ltr">Minimize to system tray</bdi> · خروج کامل: منوی کنار ساعت</span><span className="creator-credit">Created by Zexter</span><span dir="ltr">VLESS · VMess · Trojan · SS</span></footer>
    </main>
    </LayoutGroup></MotionConfig>
  );
}

export default App;
