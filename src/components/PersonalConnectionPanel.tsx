import { type FormEvent, useEffect, useRef } from "react";
import { AnimatePresence, motion } from "motion/react";
import type { ProxyProfile } from "../types";
import { collapseVariants, transitions } from "../motion";
import { SpinnerIcon } from "./Icons";

interface PersonalConnectionPanelProps {
  profile: ProxyProfile;
  loading: boolean;
  busy: boolean;
  connected: boolean;
  remember: boolean;
  saved: boolean;
  notice: string;
  showSecret: boolean;
  configInvalid: boolean;
  hasConfig: boolean;
  unsupportedConfig: boolean;
  detectedProtocol: string | null;
  confirmForget: boolean;
  onSubmit: (event: FormEvent) => void;
  onProfileChange: <K extends keyof ProxyProfile>(key: K, value: ProxyProfile[K]) => void;
  onConfigBlur: () => void;
  onShowSecretChange: (value: boolean) => void;
  onRememberChange: (value: boolean) => void;
  onSave: () => void;
  onConfirmForgetChange: (value: boolean) => void;
  onForget: () => void;
}

export function PersonalConnectionPanel({
  profile,
  loading,
  busy,
  connected,
  remember,
  saved,
  notice,
  showSecret,
  configInvalid,
  hasConfig,
  unsupportedConfig,
  detectedProtocol,
  confirmForget,
  onSubmit,
  onProfileChange,
  onConfigBlur,
  onShowSecretChange,
  onRememberChange,
  onSave,
  onConfirmForgetChange,
  onForget,
}: PersonalConnectionPanelProps) {
  const confirmButton = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (confirmForget) confirmButton.current?.focus();
  }, [confirmForget]);

  return (
    <motion.form id="connection-form" className="panel" noValidate onSubmit={onSubmit} layout>
      <div className="panel-heading">
        <div><h2>کانفیگ شخصی</h2><p>لینک اتصال خودتان را وارد کنید.</p></div>
        <AnimatePresence initial={false} mode="popLayout">
          <motion.span className="protocol-pill" key={detectedProtocol || "xray"} initial={{ opacity: 0, y: -3 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} transition={transitions.micro} aria-live="polite">
            {detectedProtocol || "Xray"}
          </motion.span>
        </AnimatePresence>
      </div>

      <label htmlFor="profile-name">
        <span>نام پروفایل</span>
        <input id="profile-name" required disabled={loading || busy || connected} value={profile.name} onChange={(event) => onProfileChange("name", event.target.value)} autoComplete="off" />
      </label>
      <label htmlFor="config-link">
        <span>لینک اتصال</span>
        <input
          id="config-link"
          disabled={loading || busy || connected}
          dir="ltr"
          type={showSecret ? "text" : "password"}
          required
          aria-describedby="config-help"
          aria-invalid={configInvalid}
          placeholder="vless:// · vmess:// · trojan:// · ss://"
          value={profile.configLink}
          onChange={(event) => onProfileChange("configLink", event.target.value.trim())}
          onBlur={onConfigBlur}
          autoComplete="off"
          spellCheck={false}
        />
        <AnimatePresence initial={false} mode="popLayout">
          <motion.small id="config-help" key={configInvalid ? "error" : "help"} className={configInvalid ? "field-error" : undefined} role={configInvalid ? "alert" : undefined} initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={transitions.micro}>
            {configInvalid ? "این نوع لینک پشتیبانی نمی‌شود." : <><bdi dir="ltr">VLESS، VMess، Trojan و Shadowsocks</bdi> پشتیبانی می‌شوند.</>}
          </motion.small>
        </AnimatePresence>
      </label>
      <div className="profile-tools">
        <button className="text-button" type="button" aria-pressed={showSecret} onClick={() => onShowSecretChange(!showSecret)}>{showSecret ? "پنهان کردن لینک" : "نمایش لینک"}</button>
        <AnimatePresence initial={false} mode="popLayout">
          <motion.span className={`saved-badge ${saved ? "is-saved" : ""}`} key={loading ? "loading" : saved ? "saved" : "unsaved"} initial={{ opacity: 0, y: 3 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} transition={transitions.micro}>
            {loading ? "در حال بارگذاری…" : saved ? "ذخیره شده" : "ذخیره نشده"}
          </motion.span>
        </AnimatePresence>
      </div>

      <label className="remember-option">
        <input type="checkbox" checked={remember} disabled={busy || loading || connected} onChange={(event) => onRememberChange(event.target.checked)} />
        <span>کانفیگ برای دفعات بعد ذخیره شود<small>رمزگذاری با حساب ویندوز؛ بدون ذخیره در مرورگر</small></span>
      </label>

      <div className="profile-tools profile-actions">
        <button className="text-button" type="button" disabled={loading || busy || connected || !hasConfig || unsupportedConfig} aria-busy={busy} onClick={onSave}>
          {busy && <SpinnerIcon />}<span>ذخیره تنظیمات</span>
        </button>
        {saved && <button className="text-button danger-text" type="button" disabled={busy || connected} aria-expanded={confirmForget} onClick={() => onConfirmForgetChange(!confirmForget)}>حذف کانفیگ ذخیره‌شده</button>}
      </div>

      <AnimatePresence initial={false}>
        {confirmForget && (
          <motion.div className="delete-confirm" variants={collapseVariants} initial="initial" animate="enter" exit="exit" role="group" aria-label="تأیید حذف کانفیگ">
            <p>کانفیگ ذخیره‌شده حذف شود؟ برای اتصال بعدی باید دوباره آن را وارد کنید.</p>
            <div>
              <button ref={confirmButton} className="text-button danger-text" type="button" disabled={busy} onClick={onForget}>بله، حذف شود</button>
              <button className="text-button" type="button" onClick={() => onConfirmForgetChange(false)}>انصراف</button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
      <p className="privacy-note" role="status" aria-live="polite">{notice || "لینک کانفیگ را وارد کنید."}</p>
    </motion.form>
  );
}
