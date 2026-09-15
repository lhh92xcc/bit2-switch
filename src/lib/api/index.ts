export type { AppId } from "./types";
export { piApi } from "./pi";
export { providersApi, universalProvidersApi } from "./providers";
export { settingsApi } from "./settings";
export { backupsApi } from "./settings";
export { mcpApi } from "./mcp";
export { profilesApi } from "./profiles";
export { promptsApi } from "./prompts";
export { skillsApi } from "./skills";
export { usageApi } from "./usage";
export { subscriptionApi } from "./subscription";
export { vscodeApi } from "./vscode";
export { proxyApi } from "./proxy";
export { openclawApi } from "./openclaw";
export { sessionsApi } from "./sessions";
export { workspaceApi } from "./workspace";
import { invoke } from "@tauri-apps/api/core";

export interface Bit2Status {
  connected: boolean;
  baseUrl: string | null;
}

export const bit2Api = {
  async openLoginWindow(): Promise<void> { await invoke("bit2_open_login_window"); },
  async cancelLogin(): Promise<void> { await invoke("bit2_cancel_login"); },
  async status(): Promise<Bit2Status> { return await invoke("bit2_status"); },
  async logout() { await invoke("bit2_logout"); },
};
export * as configApi from "./config";
export * as authApi from "./auth";
export * as copilotApi from "./copilot";
export type { ProviderSwitchEvent } from "./providers";
export type { Prompt } from "./prompts";
export type { Profile, ProfilePayload, ProfilesResponse } from "./profiles";
export type {
  CopilotDeviceCodeResponse,
  CopilotAuthStatus,
  GitHubAccount,
} from "./copilot";
export type {
  ManagedAuthProvider,
  ManagedAuthAccount,
  ManagedAuthStatus,
  ManagedAuthDeviceCodeResponse,
} from "./auth";
