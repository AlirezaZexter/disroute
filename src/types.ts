export type ConnectionStatus = "disconnected" | "connecting" | "connected" | "error";

export interface ProxyProfile {
  name: string;
  configLink: string;
}

export interface AppStatus {
  status: ConnectionStatus;
  engineReady: boolean;
  isElevated: boolean;
  message: string;
}

export type CommunitySourceKind = "githubRaw" | "githubRelease" | "subscription" | "localFile" | "url";

export interface CommunitySource {
  id: string;
  name: string;
  location: string;
  attribution: string;
  kind: CommunitySourceKind;
  enabled: boolean;
  refreshIntervalMinutes: number;
  timeoutSeconds: number;
  expectedSha256?: string | null;
  redistributionAuthorized: boolean;
  lastSuccessfulRefresh?: number | null;
  lastError?: string | null;
}

export interface CommunityCandidate {
  id: string;
  sourceId: string;
  sourceName: string;
  attribution: string;
  protocol: string;
  country?: string | null;
  supportsUdp?: boolean | null;
  addedAt?: number | null;
  expiresAt?: number | null;
}

export interface CommunitySnapshot {
  sources: CommunitySource[];
  candidates: CommunityCandidate[];
  stale: boolean;
  refreshedAt?: number | null;
  acknowledgedWarning: boolean;
  automaticFailover: boolean;
}

export interface HealthResult {
  candidateId: string;
  sourceId: string;
  sourceName: string;
  attribution: string;
  protocol: string;
  country?: string | null;
  working: boolean;
  medianLatencyMs?: number | null;
  jitterMs?: number | null;
  failureRate: number;
  udpAvailable?: boolean | null;
  label: "Working" | "Fast" | "Unstable" | "UDP unavailable" | "Untested" | "Offline";
  score: number;
  error?: string | null;
}
