export type ConnectionStatus = "disconnected" | "connecting" | "connected" | "error";

export interface ProxyProfile {
  name: string;
  endpoint: string;
  username: string;
  password: string;
  useTls: boolean;
  tlsServerName: string;
}

export interface AppStatus {
  status: ConnectionStatus;
  engineReady: boolean;
  isElevated: boolean;
  message: string;
}

