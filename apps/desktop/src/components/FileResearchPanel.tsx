import { Button } from "./controls";
import { useEffect, useState } from "react";
import { request, errorText, type Snapshot, type Reply } from "../api";
import type { ImportedFile } from "../research";
import type { Perform } from "./ui";

export default function FileResearchPanel({
  file,
  data,
  analysisId,
  perform,
  busy,
}: {
  file?: ImportedFile;
  data: Snapshot;
  analysisId?: string;
  perform: Perform;
  busy: boolean;
}) {
  const [note, setNote] = useState("");
  const [section, setSection] = useState("summary");
  const [preview, setPreview] = useState("");
  const [consent, setConsent] = useState(false);
  const [provider, setProvider] = useState(
    data.settings.default_provider_id ??
      data.providers.find((p) => p.enabled)?.id ??
      "",
  );
  const [reply, setReply] = useState<Reply>();
  const [error, setError] = useState("");
  useEffect(() => {
    let active = true;
    setNote("");
    setReply(undefined);
    setError("");
    if (file)
      void request<string>("file_note", { id: file.id })
        .then((n) => {
          if (active) setNote(n);
        })
        .catch((e) => {
          if (active) setError(errorText(e));
        });
    return () => {
      active = false;
    };
  }, [file?.id]);
  useEffect(() => {
    let active = true;
    setPreview("");
    setConsent(false);
    setReply(undefined);
    if (file && analysisId)
      void request<string>("file_artifact_preview", { id: file.id, section })
        .then((text) => {
          if (active) setPreview(text);
        })
        .catch((e) => {
          if (active) setError(errorText(e));
        });
    return () => {
      active = false;
    };
  }, [file?.id, analysisId, section]);
  useEffect(() => {
    let active = true;
    void request<{ id: string }>("ai_agent_provider", { agent: "reverse" })
      .then((p) => {
        if (active && p?.id) setProvider(p.id);
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  }, []);
  const selectedProvider = data.providers.find((p) => p.id === provider);
  return (
    <aside className="panel file-research-panel">
      <h2>Research notes</h2>
      {!file ? (
        <p className="muted">Select an imported file.</p>
      ) : (
        <>
          <label htmlFor="file-research-notes">Local file notes</label>
          <textarea
            id="file-research-notes"
            value={note}
            maxLength={32768}
            onChange={(e) => setNote(e.target.value)}
            placeholder="Observations and questions; kept locally."
          />
          <Button
            className="button secondary"
            disabled={busy}
            onClick={() =>
              void perform(
                () => request("save_file_note", { id: file.id, note }),
                "Research notes saved",
              )
            }
            disabledReason={"An operation is in progress"}
          >
            Save notes
          </Button>
          {error && <p role="alert">{error}</p>}
          <h3>AI artifact review</h3>
          <p className="helper">
            No binary content is sent automatically. Choose and review the exact
            text below; paths and local notes are excluded.
          </p>
          <label>
            Artifact selection
            <select
              aria-label="Artifact selection"
              value={section}
              onChange={(e) => setSection(e.target.value)}
            >
              <option value="summary">Metadata summary</option>
              <option value="imports">First 48 imports</option>
              <option value="strings">First 24 sampled strings</option>
            </select>
          </label>
          <pre className="hex-preview artifact-preview">
            {preview || "Run an authorized analysis to prepare a preview."}
          </pre>
          <label>
            Review provider
            <select
              aria-label="Review provider"
              value={provider}
              onChange={(e) => {
                setProvider(e.target.value);
                setConsent(false);
              }}
            >
              <option value="">Choose an AI provider</option>
              {data.providers
                .filter((p) => p.enabled)
                .map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name} · {p.model}
                  </option>
                ))}
            </select>
          </label>
          {selectedProvider && (
            <p className="helper break-value">
              Destination: {selectedProvider.base_url}
            </p>
          )}
          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={consent}
              onChange={(e) => setConsent(e.target.checked)}
            />
            I approve sending this preview to the selected AI gateway. Sample
            strings may contain sensitive data.
          </label>
          <Button
            className="button"
            disabled={
              busy || !preview || !consent || !selectedProvider?.enabled
            }
            onClick={() =>
              void perform(
                async () =>
                  setReply(
                    await request<Reply>("file_explain", {
                      review: {
                        id: file.id,
                        section,
                        provider,
                        consent,
                        preview,
                        gateway: selectedProvider!.base_url,
                      },
                    }),
                  ),
                "AI review received; interpretation is unverified",
              )
            }
            disabledReason={
              busy
                ? "An operation is in progress"
                : !preview
                  ? "Generate and review a preview first"
                  : !consent
                    ? "Review the preview and grant the required approval"
                    : "Select an enabled AI provider"
            }
          >
            Explain selected artifacts
          </Button>
          {reply && (
            <div className="chat-message">
              <small>AI interpretation · unverified</small>
              <pre>{reply.text}</pre>
            </div>
          )}
          <h3>Findings & integrations</h3>
          <p className="helper">
            Finding records, decompilation and Ghidra integration are not
            connected. Static metadata and retained jobs are available.
          </p>
        </>
      )}
    </aside>
  );
}
