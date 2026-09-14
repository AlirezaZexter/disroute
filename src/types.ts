export type ConnectionStatus = "disconnected" | "connecting" | "connected" | "error";

export interface ProxyProfile {
  name: string;
  vlessLink: string;
}

export interface AppStatus {
  status: ConnectionStatus;
  engineReady: boolean;
  isElevated: boolean;
  message: string;
}
