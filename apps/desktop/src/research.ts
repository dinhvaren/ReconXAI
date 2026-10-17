import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";
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
  source_path?: string;
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
  started_at: string | null;
  timeout_ms: number;
  progress: number;
  stage: string;
  cancel_requested: boolean;
  result: unknown | null;
  error_code: string | null;
  error_message: string | null;
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
export function selectPaths(
  folder = false,
  save = false,
  fileName?: string,
): Promise<string[]> {
  return invoke("select_local_paths", {
    folder,
    save,
    fileName: fileName ?? null,
  });
}
export const localFileKinds: TargetKind[] = [
  "binary",
  "shared_library",
  "firmware",
  "apk",
];
export const jobKinds = [
  ["local_static_binary", "Local static analysis"],
  ["binary_metadata", "Binary metadata"],
  ["strings_extraction", "Strings extraction"],
  ["artifact_index", "Local artifact index"],
  ["yara_signatures", "YARA-X file signatures (requires installation)"],
  ["target_registration_review", "Target scope review"],
] as const;
export type JobKind = (typeof jobKinds)[number][0];
export const activeJob = (job: AnalysisJob) =>
  ["queued", "running"].includes(job.status);
export function useJobEvents(onJob: (job: AnalysisJob) => void) {
  const handler = useRef(onJob);
  handler.current = onJob;
  useEffect(() => {
    let closed = false;
    let cleanup: (() => void) | undefined;
    try {
      void listen<AnalysisJob>("research-job", (event) =>
        handler.current(event.payload),
      )
        .then((unlisten) => {
          if (closed) unlisten();
          else cleanup = unlisten;
        })
        .catch(() => {}); // Unit-test DOM has no native event bridge; reads still work.
    } catch {
      /* No native bridge. */
    }
    return () => {
      closed = true;
      cleanup?.();
    };
  }, []);
}
