import { motion } from "motion/react";
import { transitions } from "../motion";

const steps = [
  {
    title: "پیش‌نیاز شبکه را نصب کنید",
    copy: <p>بستهٔ <bdi dir="ltr">ProxiFyre</bdi> را از صفحهٔ انتشار بگیرید و نصب‌کنندهٔ <bdi dir="ltr">Windows Packet Filter</bdi> را یک‌بار اجرا کنید. موتورهای برنامه همراه نصب‌کنندهٔ DisRoute هستند.</p>,
  },
  {
    title: "برنامه را با دسترسی مدیر اجرا کنید",
    copy: <p>روی میان‌بُر DisRoute راست‌کلیک کنید و <bdi dir="ltr">Run as administrator</bdi> را بزنید. این دسترسی برای قانون Firewall و مسیریابی پردازش Discord لازم است.</p>,
  },
  {
    title: "روش اتصال را انتخاب کنید",
    copy: <p>در «کانفیگ شخصی» لینک <bdi dir="ltr">VLESS، VMess، Trojan یا Shadowsocks</bdi> را وارد کنید. اگر کانفیگ ندارید، «اتصال سریع رایگان» منابع فعال را آزمایش می‌کند. هشدار سرورهای شخص ثالث را پیش از استفاده بخوانید.</p>,
  },
  {
    title: "اتصال Discord را بزنید",
    copy: <p>وقتی وضعیت «متصل است» نمایش داده شد، Discord را باز کنید. اگر از قبل باز بوده و وصل نشد، <bdi dir="ltr">Restart Discord</bdi> را بزنید.</p>,
  },
  {
    title: "وویس و استریم را جداگانه بررسی کنید",
    copy: <p>پیام و تماس صوتی مسیر یکسانی ندارند. برای وویس و استریم، کانفیگ باید <bdi dir="ltr">UDP</bdi> و سرعت آپلود مناسب داشته باشد. مصرف هم‌زمان پهنای باند هم ممکن است پینگ بازی را بالا ببرد.</p>,
  },
  {
    title: "پنجره را به System tray بفرستید",
    copy: <p><bdi dir="ltr">Minimize to system tray</bdi> فقط پنجره را می‌بندد و اتصال روشن می‌ماند. برای خروج کامل، از منوی آیکون DisRoute کنار ساعت ویندوز استفاده کنید.</p>,
  },
];

export function GuideView() {
  return (
    <motion.section className="panel guide-panel" initial={{ opacity: 0, x: -10 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: 6 }} transition={transitions.normal}>
      <div className="guide-intro">
        <div><span className="section-label">شروع کار</span><h2>راه‌اندازی DisRoute</h2><p>نصب اولیه فقط یک‌بار انجام می‌شود. بعد از آن، اتصال از داخل همین برنامه در دسترس است.</p></div>
        <span className="guide-time">حدود ۳ دقیقه</span>
      </div>
      <div className="guide-steps">
        {steps.map((step, index) => (
          <motion.article key={step.title} initial={{ opacity: 0, y: 5 }} animate={{ opacity: 1, y: 0 }} transition={{ ...transitions.normal, delay: index * 0.025 }}>
            <span className="step-number">{(index + 1).toLocaleString("fa-IR")}</span>
            <div><strong>{step.title}</strong>{step.copy}</div>
          </motion.article>
        ))}
      </div>
      <div className="guide-note"><strong>آپدیت داخل برنامه</strong><p>بالای صفحه «بررسی آپدیت» را بزنید. نسخهٔ جدید پس از دانلود و بررسی امضا نصب می‌شود؛ آپدیت خودکار و بی‌صدا نیست.</p></div>
    </motion.section>
  );
}
