import { AnimatePresence, motion } from "motion/react";
import type { AppStatus } from "../types";
import { transitions } from "../motion";
import { BidirectionalRouteIcon, SpinnerIcon } from "./Icons";

type ConnectionMode = "personal" | "community";
type CommunityBusy = "" | "refresh" | "scan" | "connect";

const statusTitles: Record<AppStatus["status"], string> = {
  disconnected: "آمادهٔ اتصال",
  connecting: "در حال اتصال",
  connected: "متصل است",
  error: "خطای اتصال",
};

function connectLabel(mode: ConnectionMode, busy: boolean, communityBusy: CommunityBusy) {
  if (mode === "personal") return busy ? "در حال اتصال…" : "اتصال Discord";
  if (communityBusy === "refresh") return "دریافت منابع…";
  if (communityBusy === "scan") return "آزمایش اتصال‌ها…";
  if (communityBusy === "connect") return "در حال اتصال…";
  return "اتصال Discord";
}

interface ConnectionStatusProps {
  appStatus: AppStatus;
  mode: ConnectionMode;
  busy: boolean;
  discordBusy: boolean;
  communityBusy: CommunityBusy;
  connectDisabled: boolean;
  inGuide: boolean;
  onOpenConnection: () => void;
  onRestartDiscord: () => void;
  onDisconnect: () => void;
}

export function ConnectionStatus({
  appStatus,
  mode,
  busy,
  discordBusy,
  communityBusy,
  connectDisabled,
  inGuide,
  onOpenConnection,
  onRestartDiscord,
  onDisconnect,
}: ConnectionStatusProps) {
  const connected = appStatus.status === "connected";
  const working = busy || discordBusy || communityBusy !== "";
  const label = connectLabel(mode, busy, communityBusy);

  return (
    <motion.section
      layout="position"
      className={`status-card status-${appStatus.status}`}
      role={appStatus.status === "error" ? "alert" : "status"}
      aria-live={appStatus.status === "error" ? "assertive" : "polite"}
      aria-busy={working}
    >
      <div className="status-orb" aria-hidden="true">
        <motion.span
          key={appStatus.status}
          initial={{ scale: 0.55, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          transition={transitions.normal}
        />
      </div>
      <div className="status-copy">
        <span className="section-label">وضعیت فعلی</span>
        <div className="status-title-stack">
          <AnimatePresence initial={false} mode="popLayout">
            <motion.h2
              key={appStatus.status}
              initial={{ opacity: 0, y: 4 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -3 }}
              transition={transitions.micro}
            >
              {statusTitles[appStatus.status]}
            </motion.h2>
          </AnimatePresence>
        </div>
        <AnimatePresence initial={false} mode="popLayout">
          <motion.p key={appStatus.message} dir="auto" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={transitions.micro}>
            {appStatus.message}
          </motion.p>
        </AnimatePresence>
      </div>
      <div className="status-controls">
        <AnimatePresence initial={false} mode="wait">
          {connected ? (
            <motion.div className="actions status-actions" key="connected" initial={{ opacity: 0, y: 4 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} transition={transitions.micro}>
              <button className="button button-secondary" type="button" onClick={onRestartDiscord} disabled={busy || discordBusy} aria-busy={discordBusy}>
                {discordBusy && <SpinnerIcon />}<span>{discordBusy ? "در حال اجرا…" : "Restart Discord"}</span>
              </button>
              <button className="button button-danger" type="button" onClick={onDisconnect} disabled={busy || discordBusy} aria-busy={busy}>
                {busy && <SpinnerIcon />}<span>{busy ? "در حال قطع…" : "قطع اتصال"}</span>
              </button>
            </motion.div>
          ) : (
            <motion.div className="actions" key="connect" initial={{ opacity: 0, y: 4 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} transition={transitions.micro}>
              <button
                className="button button-primary"
                type={inGuide ? "button" : "submit"}
                form={inGuide ? undefined : mode === "personal" ? "connection-form" : "community-form"}
                disabled={inGuide ? working : connectDisabled}
                aria-busy={working}
                onClick={inGuide ? onOpenConnection : undefined}
              >
                {working && <SpinnerIcon />}<span>{inGuide ? "بازگشت به اتصال" : label}</span>
              </button>
            </motion.div>
          )}
        </AnimatePresence>
        <div className={`route-map route-${appStatus.status}`} aria-label="مسیر دوطرفهٔ شبکه بین Discord، Proxy و Internet">
          <span>Discord</span><BidirectionalRouteIcon /><span>Proxy</span><BidirectionalRouteIcon /><span>Internet</span>
        </div>
      </div>
    </motion.section>
  );
}
