import type { SchemaReview } from "../http";

const labels: Record<string, string> = {
  confirmed_documented_mismatch: "Documented mismatch",
  possible_mismatch: "Possible mismatch",
  insufficient_evidence: "Insufficient evidence",
  unsupported_validation_case: "Unsupported validation",
  format_annotation: "Format annotation",
};
export default function SchemaResults({ result }: { result: SchemaReview }) {
  return (
    <section aria-label="Offline schema observations">
      <p>
        {result.checks} checks · {result.network_requests} network requests
      </p>
      <p className="helper">
        {result.options
          ? `Format assertions: ${result.options.assert_formats ? "on" : "off"}`
          : "Format policy not recorded for this legacy result"}
      </p>
      <p className="helper">
        {result.coverage}. These observations do not establish a security
        vulnerability.
      </p>
      {!result.observations.length && (
        <p>
          No mismatch observed by the supported checks. This is not full schema
          validation.
        </p>
      )}
      {result.observations.map((o, i) => (
        <article className="http-item" key={i}>
          <strong>{labels[o.classification] ?? o.classification}</strong>
          <p>{o.message}</p>
          <details>
            <summary>Source evidence and schema location</summary>
            <p>
              Capture evidence: <code>{o.capture_evidence_id}</code>
            </p>
            <p>
              HAR: <code>{o.capture_pointer}</code>
            </p>
            {o.instance_pointer && (
              <p>
                Decoded JSON: <code>{o.instance_pointer}</code>
              </p>
            )}
            <p>
              OpenAPI evidence: <code>{o.openapi_evidence_id}</code>
            </p>
            <p>
              OpenAPI: <code>{o.openapi_pointer}</code>
            </p>
            <p>
              Rule: <code>{o.rule}</code>
            </p>
          </details>
        </article>
      ))}
    </section>
  );
}
