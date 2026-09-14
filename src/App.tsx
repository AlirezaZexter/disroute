import { FormEvent, useEffect, useState } from "react";
import { connect, disconnect, getStatus, saveProfile } from "./api";
import type { AppStatus, ProxyProfile } from "./types";

const initialProfile: ProxyProfile = {
  name: "Discord",
  endpoint: "",
  username: "",
  password: "",
  useTls: false,
  tlsServerName: "",
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

function App() {
  const [profile, setProfile] = useState(initialProfile);
  const [appStatus, setAppStatus] = useState(emptyStatus);
  const [busy, setBusy] = useState(false);
  const connected = appStatus.status === "connected";
  const statusTitle = {
    disconnected: "آمادهٔ اتصال",
    connecting: "در حال اتصال",
    connected: "Discord متصل است",
    error: "خطای اتصال",
  }[appStatus.status];

  useEffect(() => {
    getStatus().then(setAppStatus).catch((error: unknown) => {
      setAppStatus({ ...emptyStatus, status: "error", message: String(error) });
    });
  }, []);

  const update = <K extends keyof ProxyProfile>(key: K, value: ProxyProfile[K]) =>
    setProfile((current) => ({ ...current, [key]: value }));

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setAppStatus((current) => ({ ...current, status: "connecting", message: "در حال اتصال…" }));
    try {
      await saveProfile(profile);
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
    <main className="shell">
      <header className="topbar">
        <div className="brand-mark"><ShieldIcon /></div>
        <div>
          <h1>DisRoute</h1>
          <p>مسیر اختصاصی Discord</p>
        </div>
        <span className="version">نسخهٔ آزمایشی ۰.۱</span>
      </header>

      <section className={`status-card status-${appStatus.status}`} aria-live="polite">
        <div className="status-orb"><span /></div>
        <div className="status-copy">
          <span className="eyebrow">وضعیت اتصال</span>
          <h2>{statusTitle}</h2>
          <p>{appStatus.message}</p>
        </div>
        <div className="route-map" aria-label="مسیر شبکه">
          <span>Discord</span><i /><span>Proxy</span><i /><span>Internet</span>
        </div>
      </section>

      <div className="content-grid">
        <form className="panel" onSubmit={handleSubmit}>
          <div className="panel-heading">
            <div><span className="eyebrow">پروفایل اتصال</span><h2>مشخصات SOCKS5</h2></div>
            <span className="protocol-pill">TCP + UDP</span>
          </div>

          <label>
            <span>نام پروفایل</span>
            <input value={profile.name} onChange={(e) => update("name", e.target.value)} autoComplete="off" />
          </label>
          <label>
            <span>آدرس و پورت</span>
            <input dir="ltr" required placeholder="proxy.example.com:1080" value={profile.endpoint} onChange={(e) => update("endpoint", e.target.value)} autoComplete="off" />
            <small>سرور باید SOCKS5 UDP ASSOCIATE را پشتیبانی کند.</small>
          </label>
          <div className="field-row">
            <label><span>نام کاربری</span><input dir="ltr" value={profile.username} onChange={(e) => update("username", e.target.value)} autoComplete="username" /></label>
            <label><span>رمز عبور</span><input dir="ltr" type="password" value={profile.password} onChange={(e) => update("password", e.target.value)} autoComplete="current-password" /></label>
          </div>
          <label className="toggle-row">
            <span><strong>SOCKS5 روی TLS</strong><small>در صورت پشتیبانی سرور فعال کنید</small></span>
            <input type="checkbox" checked={profile.useTls} onChange={(e) => update("useTls", e.target.checked)} />
          </label>
          {profile.useTls && <label><span>نام سرور TLS</span><input dir="ltr" required value={profile.tlsServerName} onChange={(e) => update("tlsServerName", e.target.value)} /></label>}

          <div className="actions">
            {connected ? (
              <button className="button button-danger" type="button" onClick={handleDisconnect} disabled={busy}>قطع اتصال</button>
            ) : (
              <button className="button button-primary" type="submit" disabled={busy || !profile.endpoint}>{busy ? "لطفاً صبر کنید…" : "اتصال Discord"}</button>
            )}
          </div>
          <p className="privacy-note">رمز عبور در نسخهٔ فعلی ذخیره نمی‌شود و پس از بستن برنامه از حافظه پاک خواهد شد.</p>
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
            <p>برای اجرای واقعی، بستهٔ تأییدشدهٔ موتور و درایور Windows Packet Filter لازم است.</p>
          </div>
        </aside>
      </div>
    </main>
  );
}

export default App;
