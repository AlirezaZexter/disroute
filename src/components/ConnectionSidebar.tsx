import type { AppStatus } from "../types";

export function ConnectionSidebar({ appStatus }: { appStatus: AppStatus }) {
  return (
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
        <p>{appStatus.isElevated ? "دسترسی Administrator فعال است." : "برای اتصال، برنامه را با Run as administrator اجرا کنید."}</p>
      </div>
    </aside>
  );
}
