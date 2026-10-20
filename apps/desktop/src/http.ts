import type { AnalysisJob } from "./research";
export interface HttpImport {
  id: string;
  project_id: string;
  target_id: string | null;
  kind: string;
  title: string;
  sha256: string;
  created_at: string;
  evidence_id: string;
  summary: Record<string, unknown>;
}
export interface HttpMessage {
  path_parameters?: string[];
  headers: { name: string; value: string }[];
  cookies: { name: string; value: string }[];
  body: {
    mime_type: string;
    decoded_size: number;
    preview: unknown;
    encoded: boolean;
  };
  http_version: string | null;
}
export interface HttpExchange {
  id: string;
  import_id: string;
  project_id: string;
  target_id: string | null;
  method: string;
  url: string;
  host: string;
  path: string;
  status: number;
  started_at: string;
  elapsed_ms: number;
  identity: string;
  role: string;
  authentication: string[];
  query_parameters: string[];
  request: HttpMessage;
  response: HttpMessage;
  timings: Record<string, number>;
  body_sha256: string;
}
export interface ApiOperation {
  id: string;
  import_id: string;
  target_id?: string | null;
  method: string;
  path: string;
  summary: string;
  servers: unknown;
  parameters: unknown;
  security: unknown;
  request_body: unknown;
  responses: unknown;
}
export interface SchemaObservation {
  classification: string;
  rule: string;
  message: string;
  capture_pointer: string;
  instance_pointer: string;
  openapi_pointer: string;
  capture_evidence_id: string;
  openapi_evidence_id: string;
}
export interface SchemaReview {
  options?: { assert_formats: boolean };
  observations: SchemaObservation[];
  checks: number;
  coverage: string;
  network_requests: number;
}
export interface HttpComparison {
  id: string;
  evidence_id: string;
  left_identity: string;
  right_identity: string;
  left_status: number;
  right_status: number;
  status: string;
  observations: string[];
}
export interface HttpWorkspace {
  imports: HttpImport[];
  traffic: HttpExchange[];
  operations: ApiOperation[];
  endpoints: {
    target_id: string | null;
    host: string;
    origin: string;
    method: string;
    path: string;
    requests: number;
    documented: boolean;
    observation: string;
  }[];
  total: number;
  offset: number;
  page_size: number;
  comparisons: HttpComparison[];
  analysis_jobs: { exchange_id: string; job: AnalysisJob }[];
  authorization_matrix: {
    target_id: string | null;
    origin: string;
    method: string;
    path: string;
    identity: string;
    role: string;
    capture_count: number;
    observed_paths: number;
    statuses: number[];
    body_variants: number;
    capture_ids: string[];
  }[];
}
