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
  hideToTray,
  loadProfile,
  refreshCommunity,
  restartDiscord,
  saveCommunitySources,
  saveProfile,
  scanCommunity,
  setCommunityPreferences,
} from "./api";
import type { AppStatus, CommunitySnapshot, CommunitySource, HealthResult, ProxyProfile } from "./types";
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
  }, []);

  useEffect(() => {
    if (busy || discordBusy || !connected) return;
    const timer = window.setInterval(() => { getStatus().then(setAppStatus).catch(() => {}); }, 5000);
    return () => window.clearInterval(timer);
  }, [busy, discordBusy, connected]);

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
    setBusy(true);
    setAppStatus((current) => ({ ...current, status: "connecting", message: "در حال اتصال…" }));
    try {
      await persist();
      setAppStatus(await connect(profile));
    } catch (error) { setAppStatus((current) => ({ ...current, status: "error", message: String(error) })); }
    finally { setBusy(false); }
  }

  async function handleDisconnect() {
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

  const connectDisabled = connectionMode === "personal"
    ? loading || busy || !hasConfig || unsupportedConfig
    : communityBusy !== "" || !community.acknowledgedWarning || !community.sources.some((source) => source.enabled);

  return {
    profile, appStatus, busy, discordBusy, loading, remember, saved, notice, showSecret, confirmForget,
    view, connectionMode, community, communityResults, communityBusy, communityError, warningChecked,
    sourceDraft, updatePhase, updateInfo, updateProgress, updateError, connected, detectedProtocol,
    hasConfig, unsupportedConfig, configInvalid, connectDisabled,
    setView, setConnectionMode, setShowSecret, setConfirmForget, setWarningChecked, setSourceDraft,
    setConfigTouched, setUpdatePhase, updateProfile, handleRememberChange, handleSave, handleForget,
    handleSubmit, handleDisconnect, acknowledgeCommunity, updateFailover, addSource, replaceSources,
    handleCommunityRefresh, handleCommunityScan, handleCommunityConnect, handleClearCommunityData,
    handleRestartDiscord, handleCheckUpdates, handleInstallUpdate,
    cancelCommunityScan: () => cancelCommunityScan().catch((error) => setCommunityError(String(error))),
    hideToTray: () => hideToTray().catch((error) => setNotice(String(error))),
  };
}
