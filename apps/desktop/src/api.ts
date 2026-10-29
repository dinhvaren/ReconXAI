import { invoke, Channel } from "@tauri-apps/api/core";
import type { Target, AnalysisJob } from "./research";
export type Mode = "passive_auto" | "authorized_auto" | "lab_research";
export type AppTheme = "light" | "dark" | "reconx";
export interface Project {
  id: string;
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
  archived: boolean;
}
export type Kind =
  | "openai"
  | "anthropic"
  | "gemini"
  | "deepseek"
  | "openrouter"
  | "ollama"
  | "lm_studio"
  | "compatible";
export interface Provider {
  id: string;
  name: string;
  kind: Kind;
  base_url: string;
  model: string;
  enabled: boolean;
  timeout_secs: number;
  max_output_tokens: number;
  requires_key: boolean;
}
export interface Settings {
  security_mode: Mode;
  default_provider_id: string | null;
  fallback_provider_id: string | null;
  theme: AppTheme;
}
export interface Audit {
  id: number;
  action: string;
  entity_id: string;
  outcome: string;
  timestamp: string;
}
export interface Snapshot {
  targets?: Target[];
  jobs?: AnalysisJob[];
  projects: Project[];
  providers: Provider[];
  settings: Settings;
  audit: Audit[];
  database_path: string;
  version: string;
}
export interface Reply {
  text: string;
  model: string;
  provider_id: string;
  fallback_used: boolean;
  latency_ms: number;
  input_tokens: number | null;
  output_tokens: number | null;
}
export async function request<T>(
  op: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  return invoke<T>("service", { request: { op, ...args } });
}
export async function streamChat(
  id: string,
  prompt: string,
  requestId: string,
  onUpdate: (text: string) => void,
): Promise<Reply> {
  const updates = new Channel<string>();
  updates.onmessage = onUpdate;
  return invoke<Reply>("stream_chat", { id, prompt, requestId, updates });
}
export async function cancelStream(requestId: string): Promise<void> {
  return invoke("cancel_stream", { requestId });
}
export interface FileImportProgress {
  stage: string;
  completed: number;
  total: number;
}
export async function streamImport<T>(
  project: string,
  path: string,
  authorized: boolean,
  folder: boolean,
  maxBytes: number,
  requestId: string,
  onProgress: (p: FileImportProgress) => void,
): Promise<T> {
  const updates = new Channel<FileImportProgress>();
  updates.onmessage = onProgress;
  return invoke("import_file_stream", {
    input: {
      project,
      path,
      authorized,
      folder,
      limits: { max_bytes: maxBytes },
    },
    requestId,
    updates,
  });
}
export interface ToolManifest {
  id: string;
  name: string;
  category: string;
  platform: string;
  version: string;
  source: string;
  url: string;
  sha256: string;
  binary_sha256: string;
  requires_admin: boolean;
  capabilities: string[];
  dependencies: string[];
}
export interface ToolInventory {
  manifest: ToolManifest;
  status: string;
  integrity_verified: boolean;
  receipt: { installed_at: string; last_health_check: string | null } | null;
}
export interface ToolProgress {
  phase: string;
  downloaded: number;
  total: number | null;
}
export async function installTool(
  id: string,
  onUpdate: (progress: ToolProgress) => void,
): Promise<ToolInventory> {
  const updates = new Channel<ToolProgress>();
  updates.onmessage = onUpdate;
  return invoke("install_tool", { id, updates });
}
export async function installNativeExtension(
  approval: { id: string; fingerprint: string; approved: boolean },
  onProgress: (p: ToolProgress) => void,
) {
  const updates = new Channel<ToolProgress>();
  updates.onmessage = onProgress;
  return invoke("install_native_extension", { approval, updates });
}
export function errorText(error: unknown): string {
  if (typeof error === "object" && error && "message" in error)
    return String(error.message);
  return typeof error === "string" ? error : "The operation failed.";
}
export const providerKinds: Record<
  Kind,
  { label: string; url: string; key: boolean }
> = {
  openai: { label: "OpenAI", url: "https://api.openai.com/v1", key: true },
  anthropic: {
    label: "Anthropic",
    url: "https://api.anthropic.com/v1",
    key: true,
  },
  gemini: {
    label: "Google Gemini",
    url: "https://generativelanguage.googleapis.com/v1beta",
    key: true,
  },
  deepseek: {
    label: "DeepSeek",
    url: "https://api.deepseek.com/v1",
    key: true,
  },
  openrouter: {
    label: "OpenRouter",
    url: "https://openrouter.ai/api/v1",
    key: true,
  },
  ollama: { label: "Ollama", url: "http://127.0.0.1:11434", key: false },
  lm_studio: {
    label: "LM Studio",
    url: "http://127.0.0.1:1234/v1",
    key: false,
  },
  compatible: { label: "Compatible gateway", url: "https://", key: true },
};
