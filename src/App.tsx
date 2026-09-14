import { FormEvent, useEffect, useState } from "react";
import { connect, disconnect, getStatus, saveProfile, loadProfile, forgetProfile, hideToTray } from "./api";
import { AnimatePresence, LayoutGroup, MotionConfig, motion, useReducedMotion, useIsPresent } from "motion/react";
import type { AppStatus, ProxyProfile } from "./types";

const initialProfile: ProxyProfile = {
  name: "Discord",
  vlessLink: "",
};

const emptyStatus: AppStatus = {
  status: "disconnected",
  engineReady: false,
  isElevated: false,
  message: "در حال بررسی موتور…",
};

function ShieldIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 2 4.5 5v5.7c0 4.8 3 9.2 7.5 11.3 4.5-2.1 7.5-6.5 7.5-11.3V5L12 2Z" />
      <path d="m8.8 12 2 2 4.4-4.5" />
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
  const [loading, setLoading] = useState(true);
  const [remember, setRemember] = useState(true);
  const [saved, setSaved] = useState(false);
  const [notice, setNotice] = useState("");
  const [showSecret, setShowSecret] = useState(false);
  const [confirmForget, setConfirmForget] = useState(false);
  const [view, setView] = useState<"connection" | "guide">("connection");
  const reduced = useReducedMotion();
  const connected = appStatus.status === "connected";
  const statusTitle = {
    disconnected: "آمادهٔ اتصال",
    connecting: "در حال اتصال",
    connected: "تونل آماده است",
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
  }, []);

  useEffect(() => {
    if (busy || !connected) return;
    const timer = window.setInterval(() => { getStatus().then(setAppStatus).catch(() => {}); }, 5000);
    return () => window.clearInterval(timer);
  }, [busy, connected]);

  const update = <K extends keyof ProxyProfile>(key: K, value: ProxyProfile[K]) =>
    { setProfile((current) => ({ ...current, [key]: value })); setNotice("تغییرات هنوز ذخیره نشده‌اند."); };

  async function persist() {
    if (remember) { await saveProfile(profile); setSaved(true); setNotice("برای اتصال بعدی، امن در ویندوز ذخیره شد."); }
    else { await forgetProfile(); setSaved(false); setNotice("کانفیگ فقط برای همین اجرا نگه داشته می‌شود."); }
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

  return (
    <MotionConfig reducedMotion="user" transition={{ duration: reduced ? 0 : .22 }}><LayoutGroup>
    <main className="shell">
      <header className="topbar">
        <div className="brand-mark"><ShieldIcon /></div>
        <div>
          <h1>DisRoute</h1>
          <p>مسیر اختصاصی Discord</p>
        </div>
        <span className="version">WINDOWS · 0.2.1 PREVIEW</span>
        <button className="text-button" type="button" title="اتصال فعال می‌ماند؛ از آیکون کنار ساعت دوباره باز کنید" onClick={() => hideToTray().catch((error) => setNotice(String(error)))}>کنار ساعت</button>
      </header>

      <div className="workspace-heading"><div><span className="eyebrow">YOUR DISCORD. YOUR ROUTE.</span><h2>وصل بمان. با مسیر خودت.</h2></div><span className="local-badge">ذخیره روی همین دستگاه</span></div>
      <nav className="view-switch" aria-label="بخش‌های برنامه">
        {([['connection', 'اتصال و پروفایل'], ['guide', 'راهنمای شروع']] as const).map(([id, label]) => <button type="button" key={id} aria-pressed={view === id} onClick={() => setView(id)}>{view === id && <motion.span className="selected-view" layoutId={reduced ? undefined : 'selected-view'} transition={{ type: 'spring', stiffness: 380, damping: 32 }} />}<span>{label}</span></button>)}
      </nav>

      <section
        className={`status-card status-${appStatus.status}`}
        role={appStatus.status === "error" ? "alert" : "status"}
        aria-live={appStatus.status === "error" ? "assertive" : "polite"}
      >
        <div className="status-orb"><span /></div>
        <div className="status-copy">
          <span className="eyebrow">وضعیت اتصال</span>
          <div className="status-title-stack"><AnimatePresence initial={false}><StatusHeading key={statusTitle} title={statusTitle} reduced={!!reduced} /></AnimatePresence></div>
          <p>{appStatus.message}</p>
        </div>
        <div className="status-controls"><div className="actions">
            {connected ? (
              <button className="button button-danger" type="button" onClick={handleDisconnect} disabled={busy}>قطع اتصال</button>
            ) : (
              <button className="button button-primary" type="submit" form="connection-form" disabled={loading || busy || !profile.vlessLink}>{busy ? "لطفاً صبر کنید…" : "اتصال Discord"}</button>
            )}
          </div><div className="route-map" aria-label="مسیر شبکه">
          <span>Discord</span><i /><span>Proxy</span><i /><span>Internet</span>
        </div></div>
      </section>

      <div className="content-grid" hidden={view !== 'connection'}>
        <form id="connection-form" className="panel" onSubmit={handleSubmit}>
          <div className="panel-heading">
            <div><span className="eyebrow">پروفایل اتصال</span><h2>مشخصات VLESS</h2></div>
            <span className="protocol-pill">VLESS</span>
          </div>

          <label>
            <span>نام پروفایل</span>
            <input required disabled={loading || busy || connected} value={profile.name} onChange={(e) => update("name", e.target.value)} autoComplete="off" />
          </label>
          <label>
            <span>لینک اتصال VLESS</span>
            <input disabled={loading || busy || connected} dir="ltr" type={showSecret ? "text" : "password"} required aria-describedby="vless-help" placeholder="vless://uuid@server:443?..." value={profile.vlessLink} onChange={(e) => update("vlessLink", e.target.value.trim())} autoComplete="off" spellCheck={false} />
            <small id="vless-help">Reality، TLS و انتقال‌های TCP، WebSocket و gRPC پشتیبانی می‌شوند.</small>
          </label>
          <div className="profile-tools"><button className="text-button" type="button" aria-pressed={showSecret} onClick={() => setShowSecret(!showSecret)}>{showSecret ? 'پنهان کردن لینک' : 'نمایش لینک'}</button><span className="saved-badge">{loading ? 'در حال بازیابی…' : saved ? 'پروفایل ذخیره دارد' : 'پروفایل جدید'}</span></div>
          <label className="remember-option"><input type="checkbox" checked={remember} disabled={busy || loading || connected} onChange={(e) => { setRemember(e.target.checked); setNotice('برای اعمال این انتخاب، ذخیره را بزنید.'); }} /><span>کانفیگ برای دفعات بعد ذخیره شود<small>رمزگذاری با حساب ویندوز شما؛ بدون ذخیره در مرورگر</small></span></label>

          <div className="profile-tools"><button className="text-button" type="button" disabled={loading || busy || connected || !profile.vlessLink} onClick={handleSave}>ذخیره تنظیمات</button>{saved && <button className="text-button danger-text" type="button" disabled={busy || connected} onClick={() => setConfirmForget(!confirmForget)}>حذف کانفیگ ذخیره‌شده</button>}</div>
          {confirmForget && <div className="delete-confirm"><p>کانفیگ ذخیره‌شده حذف شود؟ برای اتصال بعدی باید دوباره واردش کنید.</p><button className="text-button danger-text" type="button" disabled={busy} onClick={handleForget}>بله، حذف شود</button><button className="text-button" type="button" onClick={() => setConfirmForget(false)}>انصراف</button></div>}
          <p className="privacy-note" role="status">{notice || 'با اتصال، تنظیمات ذخیره‌سازی انتخاب‌شده اعمال می‌شوند.'}</p>
        </form>

        <aside className="panel side-panel">
          <div className="panel-heading"><div><span className="eyebrow">دامنهٔ اثر</span><h2>فقط Discord</h2></div></div>
          <ul className="app-list">
            <li className="active"><span className="app-dot discord" /><div><strong>Discord</strong><small>TCP و UDP از پروکسی</small></div><span>Proxy</span></li>
            <li><span className="app-dot game" /><div><strong>بازی‌ها</strong><small>بدون تغییر مسیر</small></div><span>Direct</span></li>
            <li><span className="app-dot browser" /><div><strong>سایر برنامه‌ها</strong><small>اینترنت عادی</small></div><span>Direct</span></li>
          </ul>
          <div className="requirement">
            <strong>{appStatus.engineReady ? "موتور آماده است" : "موتور شبکه نصب نیست"}</strong>
            <p>در اولین اتصال، DisRoute یک مجوز Firewall محدود به موتور خودش می‌سازد و سپس مسیر VLESS را واقعاً آزمایش می‌کند.</p>
            <p>{appStatus.isElevated ? 'دسترسی Administrator فعال است.' : 'برای اتصال، برنامه را با Run as administrator اجرا کنید.'}</p>
          </div>
        </aside>
      </div>
      <AnimatePresence>{view === 'guide' && <motion.section className="panel guide-panel" initial={{ opacity: 0, y: reduced ? 0 : 8 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }}><span className="eyebrow">QUICK START</span><h2>از دانلود تا اولین اتصال</h2><ol><li><strong>بسته را کامل Extract کن.</strong><p>پوشه engine را کنار برنامه نگه دار. در اولین نصب، پیش‌نیازهای معرفی‌شده در راهنمای بسته لازم‌اند.</p></li><li><strong>با دسترسی Administrator اجرا کن.</strong><p>لینک VLESS خودت را وارد کن؛ گزینه ذخیره، ورود دوباره در دفعات بعد را حذف می‌کند.</p></li><li><strong>متصل شو و Discord را باز کن.</strong><p>آماده‌بودن تونل به معنی تأیید صدای دوطرفه نیست. سرور VLESS باید UDP را پشتیبانی کند.</p></li></ol><div className="requirement"><strong>برای دوستانت فقط ZIP نسخه را بفرست.</strong><p>هر نفر کانفیگ خودش را وارد می‌کند. فایل‌های AppData و گزارش‌ها را به بسته اضافه نکن.</p></div></motion.section>}</AnimatePresence>
      <footer className="app-footer"><span>بستن پنجره: ادامه کنار ساعت · خروج کامل: منوی آیکون</span><span>VLESS · TCP + UDP</span></footer>
    </main>
    </LayoutGroup></MotionConfig>
  );
}

export default App;
