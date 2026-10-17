import type { Provider } from "./api";
export interface ModelPreset {
  id: string;
  name: string;
  description: string;
  capabilities: {
    tool_calling: boolean | null;
    vision: boolean | null;
    reasoning: boolean | null;
    streaming: boolean | null;
    structured_output: boolean | null;
    context_window: number | null;
  };
  deprecated: boolean | null;
}
export interface ProviderPreset {
  id: string;
  name: string;
  category: string;
  kind: Provider["kind"];
  base_url: string;
  endpoint_template?: string | null;
  requires_key: boolean;
  discovery: boolean;
  setup_note: string;
  docs_url: string;
  verified_on: string;
  models: ModelPreset[];
}
export interface GatewayOptions {
  auth_header: string | null;
  auth_prefix: string;
  proxy_url: string | null;
  discovery_path: string | null;
  streaming: boolean;
}
export const defaultOptions: GatewayOptions = {
  auth_header: null,
  auth_prefix: "Bearer ",
  proxy_url: null,
  discovery_path: null,
  streaming: true,
};
export interface AiProfile {
  provider: Provider;
  vendor_id: string;
  options: GatewayOptions;
  header_names: string[];
  credential_state: string;
  key_configured?: boolean;
  connection: {
    state: string;
    checked_at: string;
    last_success_at: string | null;
    latency_ms: number;
    error_code: string | null;
    message: string;
  } | null;
  models: string[];
  models_refreshed_at: string | null;
}
export interface AgentRoute {
  agent_id: string;
  provider_id: string;
}
export interface AiManagerState {
  catalog: ProviderPreset[];
  profiles: AiProfile[];
  routing: AgentRoute[];
  agents: string[];
}
