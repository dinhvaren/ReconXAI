import type { ToolInventory } from "./api";
import type { AnalysisJob } from "./research";
export interface ManagedMcpSummary {
  server_id: string | null;
  active_job: AnalysisJob | null;
  tools: {
    name: string;
    description?: string;
    annotations?: {
      readOnlyHint?: boolean;
      destructiveHint?: boolean;
      openWorldHint?: boolean;
    };
  }[];
  allowed_tools: string[];
  package_name: string;
  runtime_requirement: string;
}
export interface ExtensionState {
  version: string;
  enabled: boolean;
  status: string;
  revision: string;
  updated_at: string;
  last_health_check: string | null;
  error_code: string | null;
}
export interface ExtensionManifest {
  manifest_version: number;
  id: string;
  name: string;
  publisher: string;
  version: string;
  description: string;
  extension_type: string;
  category: string;
  compatible_reconx: string;
  operating_systems: string[];
  architectures: string[];
  required_runtime: string;
  required_tools: string[];
  required_mcp_servers: string[];
  install_source: string;
  permissions: string[];
  capabilities: string[];
  configuration_schema: unknown;
  entrypoint: string;
  health_check: string;
  uninstall_method: string;
}
export interface ExtensionEntry {
  manifest: ExtensionManifest;
  provenance: string;
  status: string;
  supported: boolean;
  installed: ExtensionState | null;
  dependencies: ToolInventory[];
  update_available: boolean;
  available_version: string;
  installation_size: number | null;
  source_repository: string;
  managed?: ManagedMcpSummary | null;
}
export interface ExtensionPlan {
  extension: ExtensionEntry;
  fingerprint: string;
  actions: string[];
  requires_admin: boolean;
  network_access: string;
  package_plan?: {
    version: string;
    runtime: {
      name: string;
      version: string;
      managed: boolean;
      executable: string;
      source: string | null;
      sha256: string;
      archive_sha256: string | null;
    };
    dependencies: {
      path: string;
      version: string;
      source: string;
      integrity: string;
      dependencies: string[];
    }[];
    minimum_free_bytes: number;
    lock_sha256: string;
  } | null;
}
