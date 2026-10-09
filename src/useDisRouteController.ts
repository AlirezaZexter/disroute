import { type FormEvent, useEffect, useRef, useState } from "react";
import type { Update } from "@tauri-apps/plugin-updater";
import {
  cancelCommunityScan,
  clearCommunityData,
  connect,
  connectCommunity,
  disconnect,
  forgetProfile,
  getCommunitySnapshot,
  getStatus,
  getConnectionHealth,
  checkConnectionHealth,
  checkPrerequisites,
  installPrerequisites,
  hideToTray,
  loadProfile,
  refreshCommunity,
  restartDiscord,
  saveCommunitySources,
  saveProfile,
  scanCommunity,
  setCommunityPreferences,
} from "./api";
import type { AppStatus, ConnectionHealth, CommunitySnapshot, CommunitySource, HealthResult, ProxyProfile, PrerequisiteStatus } from "./types";
import { downloadAndInstall, findUpdate, type UpdateProgress } from "./updater";
import type { AppView, UpdatePhase } from "./components/AppChrome";
import type { CommunityBusy, SourceDraft } from "./components/CommunityConnectionPanel";

const initialProfile: ProxyProfile = { name: "Discord", configLink: "" };
const initialSourceDraft: SourceDraft = {
  name: "",
  location: "",
  attribution: "",
  kind: "url",
  refreshIntervalMinutes: 60,
  timeoutSeconds: 12,
  expectedSha256: "",
  redistributionAuthorized: false,
};
const emptyStatus: AppStatus = { status: "disconnected", engineReady: false, isElevated: false, message: "در حال بررسی موتور…" };
const emptyCommunity: CommunitySnapshot = { sources: [], candidates: [], stale: false, acknowledgedWarning: false, automaticFailover: true };
const protocolNames = { vless: "VLESS", vmess: "VMess", trojan: "Trojan", ss: "Shadowsocks" } as const;

function detectProtocol(value: string) {
  const scheme = value.trim().match(/^([a-z][a-z0-9+.-]*):\/\//i)?.[1]?.toLowerCase();
  return scheme && scheme in protocolNames ? protocolNames[scheme as keyof typeof protocolNames] : null;
}

function sourceId() {
  return `source-${Date.now().toString(36)}`;
}

export function useDisRouteController() {
  const [profile, setProfile] = useState(initialProfile);
  const [appStatus, setAppStatus] = useState(emptyStatus);
  const [connectionHealth, setConnectionHealth] = useState<ConnectionHealth | null>(null);
  const [healthBusy, setHealthBusy] = useState(false);
  const [healthError, setHealthError] = useState("");
  const healthEpoch = useRef(0);
  const [busy, setBusy] = useState(false);
  const [discordBusy, setDiscordBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const [remember, setRemember] = useState(true);
  const [saved, setSaved] = useState(false);
  const [notice, setNotice] = useState("");
  const [showSecret, setShowSecret] = useState(false);
  const [configTouched, setConfigTouched] = useState(false);
  const [confirmForget, setConfirmForget] = useState(false);
  const [view, setView] = useState<AppView>("connection");
  const [connectionMode, setConnectionMode] = useState<"personal" | "community">("personal");
  const [community, setCommunity] = useState<CommunitySnapshot>(emptyCommunity);
  const [communityResults, setCommunityResults] = useState<HealthResult[]>([]);
  const [communityBusy, setCommunityBusy] = useState<CommunityBusy>("");
  const [communityError, setCommunityError] = useState("");
  const [warningChecked, setWarningChecked] = useState(false);
  const [sourceDraft, setSourceDraft] = useState<SourceDraft>(initialSourceDraft);
  const [updatePhase, setUpdatePhase] = useState<UpdatePhase>("idle");
  const [updateInfo, setUpdateInfo] = useState<{ version: string; notes?: string } | null>(null);
  const [updateProgress, setUpdateProgress] = useState<UpdateProgress>({ downloaded: 0 });
  const [updateError, setUpdateError] = useState("");
  const pendingUpdate = useRef<Update | null>(null);
  const [prerequisites, setPrerequisites] = useState<PrerequisiteStatus | null>(null);
  const [setupBusy, setSetupBusy] = useState<"" | "check" | "install">("check");
  const [setupError, setSetupError] = useState("");
  const [setupNotice, setSetupNotice] = useState("");
  const [restartRequired, setRestartRequired] = useState(false);
  const setupReady = !prerequisites || (prerequisites.supportedPlatform && prerequisites.engines && prerequisites.packetFilter && prerequisites.visualCpp && prerequisites.dotNet && prerequisites.elevated && !restartRequired);

  const connected = appStatus.status === "connected";
  const detectedProtocol = detectProtocol(profile.configLink);
  const hasConfig = profile.configLink.trim().length > 0;
  const unsupportedConfig = hasConfig && !detectedProtocol;
  const configInvalid = configTouched && unsupportedConfig;

  useEffect(() => {
    getStatus().then(setAppStatus).catch((error: unknown) => setAppStatus({ ...emptyStatus, status: "error", message: String(error) }));
    loadProfile()
      .then((value) => {
        if (value) {
          setProfile(value);
          setSaved(true);
          setNotice("پروفایل ذخیره‌شده بازیابی شد.");
        }
      })
      .catch(() => setNotice("بازیابی پروفایل ممکن نشد؛ کانفیگ را دوباره وارد و ذخیره کنید."))
      .finally(() => setLoading(false));
    getCommunitySnapshot().then(setCommunity).catch(() => setCommunityError("خواندن تنظیمات اتصال سریع ممکن نشد."));
    checkPrerequisites().then(setPrerequisites)
      .catch(() => setSetupError("بررسی پیش‌نیازها کامل نشد؛ «بررسی دوباره» را بزنید."))
      .finally(() => setSetupBusy(""));
  }, []);

  async function handleCheckPrerequisites() {
    if (setupBusy) return;
    setSetupBusy("check"); setSetupError("");
    try {
      const status = await checkPrerequisites();
      setPrerequisites(status);
      if (status && !status.pendingReboot) setRestartRequired(false);
    } catch (error) { setSetupError(String(error)); }
    finally { setSetupBusy(""); }
  }

  async function handleInstallPrerequisites() {
    if (setupBusy || connected || busy || communityBusy || updatePhase === "downloading" || updatePhase === "installing") return;
    setSetupBusy("install"); setSetupError("");
    setSetupNotice("نصب‌کننده باز می‌شود؛ مراحل آن را کامل کنید و به DisRoute برگردید.");
    try {
      const result = await installPrerequisites();
      setPrerequisites(result.status);
      setRestartRequired(result.outcome === "restartRequired");
      const messages = {
        completed: "نصب تمام شد؛ پیش‌نیازها دوباره بررسی شدند.",
        alreadyInstalled: "پیش‌نیازهای شبکه از قبل نصب هستند.",
        cancelled: "نصب لغو شد. هر وقت خواستید می‌توانید دوباره امتحان کنید.",
        restartRequired: "برای کامل‌شدن نصب، Windows را Restart کنید و DisRoute را دوباره باز کنید.",
        incomplete: "نصب کامل نشده است؛ پیام نصب‌کننده را بررسی کنید و دوباره امتحان کنید.",
      };
      setSetupNotice(messages[result.outcome]);
      if (result.outcome === "incomplete") setSetupError(messages.incomplete);
      setAppStatus(await getStatus());
    } catch (error) { setSetupError(String(error)); setSetupNotice(""); }
    finally { setSetupBusy(""); }
  }

  useEffect(() => {
    if (busy || discordBusy || communityBusy || setupBusy === "install") return;
    let cancelled = false;
    let timer: number;
    async function poll() {
      try {
        const [status, health] = await Promise.all([getStatus(), getConnectionHealth()]);
        if (!cancelled) {
          // Keep a failed connection's actionable error until a new connection or explicit action.
          setAppStatus(previous => previous.status === "error" && status.status === "disconnected"
            ? { ...status, status: "error", message: previous.message } : status);
          setConnectionHealth(health);
        }
      } catch { /* Retain the last state; its measurement becomes stale visibly. */ }
      finally { if (!cancelled) timer = window.setTimeout(poll, 5000); }
    }
    // Poll even while disconnected: background recovery may restore the engine.
    timer = window.setTimeout(poll, 1000);
    return () => { cancelled = true; window.clearTimeout(timer); };
  }, [busy, discordBusy, communityBusy, setupBusy]);

  async function handleHealthCheck() {
    if (healthBusy || busy || communityBusy || !connected) return;
    const epoch = healthEpoch.current;
    setHealthBusy(true); setHealthError("");
    try {
      const health = await checkConnectionHealth();
      if (epoch === healthEpoch.current) setConnectionHealth(health);
    } catch { if (epoch === healthEpoch.current) setHealthError("بررسی کامل نشد؛ کمی بعد دوباره امتحان کنید."); }
    finally { setHealthBusy(false); }
  }

  const updateProfile = <K extends keyof ProxyProfile>(key: K, value: ProxyProfile[K]) => {
    setProfile((current) => ({ ...current, [key]: value }));
    setSaved(false);
    setNotice("تغییرات هنوز ذخیره نشده‌اند.");
  };

  async function persist() {
    if (remember) {
      await saveProfile(profile);
      setSaved(true);
      setNotice("کانفیگ ذخیره شد.");
    } else {
      await forgetProfile();
      setSaved(false);
      setNotice("کانفیگ ذخیره نمی‌شود.");
    }
  }

  async function handleSave() {
    setBusy(true);
    try { await persist(); }
    catch (error) { setNotice(String(error)); }
    finally { setBusy(false); }
  }

  async function handleForget() {
    setBusy(true);
    try {
      await forgetProfile();
      setSaved(false);
      setRemember(false);
      setConfirmForget(false);
      setProfile(initialProfile);
      setNotice("کانفیگ ذخیره‌شده حذف شد.");
    } catch (error) { setNotice(String(error)); }
    finally { setBusy(false); }
  }

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    if (setupBusy || !setupReady || setupError) return;
    healthEpoch.current += 1;
    setHealthError("");
    setBusy(true);
    setAppStatus((current) => ({ ...current, status: "connecting", message: "در حال اتصال…" }));
    try {
      await persist();
      setAppStatus(await connect(profile));
    } catch (error) { setAppStatus((current) => ({ ...current, status: "error", message: String(error) })); }
    finally { setBusy(false); }
  }

  async function handleDisconnect() {
    healthEpoch.current += 1;
    setConnectionHealth(null); setHealthError("");
    setBusy(true);
    try { setAppStatus(await disconnect()); }
    catch (error) { setAppStatus((current) => ({ ...current, status: "error", message: String(error) })); }
    finally { setBusy(false); }
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
      setSourceDraft(initialSourceDraft);
      setCommunityError("");
    } catch (error) { setCommunityError(String(error)); }
  }

  async function replaceSources(sources: CommunitySource[]) {
    try {
      setCommunity(await saveCommunitySources(sources));
      setCommunityError("");
    } catch (error) { setCommunityError(String(error)); }
  }

  async function handleCommunityRefresh() {
    setCommunityBusy("refresh");
    setCommunityError("");
    setCommunityResults([]);
    try { setCommunity(await refreshCommunity()); }
    catch (error) { setCommunityError(String(error)); }
    finally { setCommunityBusy(""); }
  }

  async function handleCommunityScan() {
    setCommunityBusy("scan");
    setCommunityError("");
    setCommunityResults([]);
    try { setCommunityResults(await scanCommunity()); }
    catch (error) { setCommunityError(String(error)); }
    finally { setCommunityBusy(""); }
  }

  async function handleCommunityConnect(event: FormEvent) {
    event.preventDefault();
    if (setupBusy || !setupReady || setupError) return;
    if (!community.acknowledgedWarning || communityBusy) return;
    healthEpoch.current += 1;
    setHealthError("");
    setCommunityError("");
    try {
      setCommunityBusy("refresh");
      // Background refresh maintains the cache; do not download every source
      // again on every connect. An empty/stale snapshot still gets a refresh.
      const snapshot = community.candidates.length && !community.stale
        ? community : await refreshCommunity();
      setCommunity(snapshot);
      if (!snapshot.candidates.length) throw new Error("منابع فعلی کانفیگ قابل‌آزمایشی ندارند. وضعیت منابع را بررسی کنید.");
      setCommunityBusy("scan");
      const results = await scanCommunity();
      setCommunityResults(results);
      const healthy = results.filter((result) => result.working).map((result) => result.candidateId);
      if (!healthy.length) throw new Error("در این آزمایش هیچ اتصال فعالی به Discord پیدا نشد. بعداً دوباره امتحان کنید.");
      setCommunityBusy("connect");
      setAppStatus((current) => ({ ...current, status: "connecting", message: "گزینهٔ برتر آماده است؛ در حال اتصال…" }));
      setAppStatus(await connectCommunity(healthy));
    } catch (error) {
      setCommunityError(String(error));
      setAppStatus((current) => current.status === "connecting" ? { ...current, status: "error", message: String(error) } : current);
    } finally { setCommunityBusy(""); }
  }

  async function handleClearCommunityData() {
    try {
      setCommunity(await clearCommunityData());
      setCommunityResults([]);
      setCommunityError("");
    } catch (error) { setCommunityError(String(error)); }
  }

  async function handleRestartDiscord() {
    setDiscordBusy(true);
    setAppStatus((current) => ({ ...current, message: "در حال راه‌اندازی مجدد Discord…" }));
    try {
      const message = await restartDiscord();
      setAppStatus((current) => ({ ...current, message }));
    } catch (error) {
      setAppStatus((current) => ({ ...current, message: `اتصال روشن است؛ راه‌اندازی مجدد Discord ناموفق بود: ${String(error)}` }));
    } finally { setDiscordBusy(false); }
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
    if (setupBusy === "install") return;
    const update = pendingUpdate.current;
    if (!update) return;
    setUpdateProgress({ downloaded: 0 });
    setUpdateError("");
    setUpdatePhase("downloading");
    try {
      await downloadAndInstall(update, setUpdateProgress, async () => {
        setUpdatePhase("installing");
        if (connected) setAppStatus(await disconnect());
      });
    } catch (error) {
      setUpdateError(`دانلود یا نصب نسخه جدید ناموفق بود: ${String(error)}`);
      setUpdatePhase("error");
    }
  }

  function handleRememberChange(value: boolean) {
    setRemember(value);
    setNotice("برای اعمال این انتخاب، ذخیره را بزنید.");
  }

  const connectDisabled = Boolean(setupBusy || !setupReady || setupError) || (connectionMode === "personal"
    ? loading || busy || !hasConfig || unsupportedConfig
    : communityBusy !== "" || !community.acknowledgedWarning || !community.sources.some((source) => source.enabled));

  return {
    profile, appStatus, connectionHealth, healthBusy, healthError, handleHealthCheck, busy, discordBusy, loading, remember, saved, notice, showSecret, confirmForget,
    view, connectionMode, community, communityResults, communityBusy, communityError, warningChecked,
    sourceDraft, updatePhase, updateInfo, updateProgress, updateError, connected, detectedProtocol,
    hasConfig, unsupportedConfig, configInvalid, connectDisabled,
    prerequisites, setupBusy, setupError, setupNotice, setupReady, handleCheckPrerequisites, handleInstallPrerequisites,
    setView, setConnectionMode, setShowSecret, setConfirmForget, setWarningChecked, setSourceDraft,
    setConfigTouched, setUpdatePhase, updateProfile, handleRememberChange, handleSave, handleForget,
    handleSubmit, handleDisconnect, acknowledgeCommunity, updateFailover, addSource, replaceSources,
    handleCommunityRefresh, handleCommunityScan, handleCommunityConnect, handleClearCommunityData,
    handleRestartDiscord, handleCheckUpdates, handleInstallUpdate,
    cancelCommunityScan: () => cancelCommunityScan().catch((error) => setCommunityError(String(error))),
    hideToTray: () => hideToTray().catch((error) => setNotice(String(error))),
  };
}
