import { invoke } from "@tauri-apps/api/core";
import type { Mode } from "./api";
export const targetKinds = [
  ["domain", "Domain"],
  ["subdomain", "Subdomain"],
  ["url", "URL / web application"],
  ["ipv4", "IPv4"],
  ["ipv6", "IPv6"],
  ["cidr", "CIDR network"],
  ["hostname", "Hostname"],
  ["rest_api", "REST API"],
  ["graphql", "GraphQL endpoint"],
  ["git_repository", "Git repository"],
  ["source_directory", "Source directory"],
  ["binary", "Binary executable"],
  ["shared_library", "Shared library"],
  ["firmware", "Firmware image"],
  ["apk", "Android APK"],
  ["container_image", "Container image"],
  ["iac_directory", "Cloud / IaC directory"],
] as const;
export type TargetKind = (typeof targetKinds)[number][0];
export interface TargetInput {
  project_id: string;
  name: string;
  kind: TargetKind;
  value: string;
  description: string;
  tags: string[];
  allowed_scope: string[];
  excluded_scope: string[];
  authorized: boolean;
  mode: Mode;
}
export interface Target extends TargetInput {
  id: string;
  scope_status: string;
  created_at: string;
  updated_at: string;
  last_analyzed_at: string | null;
}
export interface ScopeDecision {
  target_id: string;
  candidate: string;
  authorized: boolean;
  in_scope: boolean;
  status: string;
  reason: string;
}
export interface ImportedFile {
  id: string;
  project_id: string;
  target_id: string | null;
  name: string;
  sha256: string;
  size: number;
  format: string;
  created_at: string;
}
export interface AnalysisJob {
  id: string;
  project_id: string;
  target_id: string | null;
  file_id: string | null;
  kind: string;
  status: string;
  created_at: string;
  finished_at: string | null;
  error_code: string | null;
  logs: string[];
}
export interface Section {
  name: string;
  address: number;
  size: number;
  file_offset: number | null;
  entropy: number | null;
}
export interface BinaryString {
  offset: number;
  encoding: string;
  text: string;
}
export interface BinaryAnalysis {
  format: string;
  architecture: string | null;
  size: number;
  sha256: string;
  entry_point: number | null;
  entropy: number;
  sections: Section[];
  segments: Section[];
  imports: string[];
  exports: string[];
  package_entries: string[];
  strings: BinaryString[];
  hex_preview: string;
  warnings: string[];
}
export interface AnalysisRecord {
  id: string;
  file_id: string;
  job_id: string;
  created_at: string;
  analysis: BinaryAnalysis;
}
export function selectPaths(folder = false, save = false): Promise<string[]> {
  return invoke("select_local_paths", { folder, save });
}
export const localFileKinds: TargetKind[] = [
  "binary",
  "shared_library",
  "firmware",
  "apk",
];
