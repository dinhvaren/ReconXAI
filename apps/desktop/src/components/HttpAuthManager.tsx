import { Button } from "./controls";
import { useEffect, useRef, useState } from "react";
import { request, errorText, type Snapshot } from "../api";
import { Modal } from "./ui";
import type { AuthProfile } from "./HttpRequestWorkspace";

export default function HttpAuthManager({
  open,
  project,
  target,
  data,
  onClose,
  onTest,
}: {
  open: boolean;
  project: string;
  target: string;
  data: Snapshot;
  onClose: () => void;
  onTest: (p: AuthProfile) => void;
}) {
  const [profiles, setProfiles] = useState<AuthProfile[]>([]),
    [editing, setEditing] = useState<AuthProfile | null>(null),
    [targetId, setTarget] = useState(target),
    [name, setName] = useState(""),
    [role, setRole] = useState("user"),
    [kind, setKind] = useState("anonymous"),
    [header, setHeader] = useState("x-api-key"),
    [secret, setSecret] = useState(""),
    [user, setUser] = useState(""),
    [show, setShow] = useState(false),
    [error, setError] = useState(""),
    [notice, setNotice] = useState(""),
    [busy, setBusy] = useState(false);
  const version = useRef(0);
  const reset = () => {
    setEditing(null);
    setTarget(target);
    setName("");
    setRole("user");
    setKind("anonymous");
    setHeader("x-api-key");
    setSecret("");
    setUser("");
    setShow(false);
  };
  const load = async () => {
    const current = version.current;
    const rows = await request<AuthProfile[]>("http_auth_profiles", {
      project,
      target: null,
    });
    if (current === version.current) setProfiles(rows);
  };
  useEffect(() => {
    version.current++;
    const current = version.current;
    if (open) {
      reset();
      setProfiles([]);
      setError("");
      setNotice("");
      void load().catch((e) => {
        if (current === version.current) setError(errorText(e));
      });
    } else {
      setSecret("");
      setUser("");
    }
    return () => {
      version.current++;
    };
  }, [open, project, target]);
  const act = async (fn: () => Promise<void>) => {
    setBusy(true);
    setError("");
    try {
      await fn();
      await load();
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Modal
      title="HTTP authentication profiles"
      open={open}
      onClose={onClose}
      dismissDisabled={busy}
    >
      <div className="form http-dialog-content">
        <p>
          Credentials stay in OS secure storage and are bound to this project,
          target and exact origin. Saving a profile never authenticates it or
          sends a request.
        </p>
        {error && <p role="alert">{error}</p>}
        {notice && <p role="status">{notice}</p>}
        <section aria-label="Configured HTTP identities">
          <h3>Test identities</h3>
          {!profiles.length && <p>No identities configured.</p>}
          {profiles.map((p) => (
            <article key={p.id} className="http-item">
              <h3>
                {p.name} · {p.role}
              </h3>
              <p>
                {p.kind} · {p.credential_state} · <code>{p.origin}</code>
              </p>
              <p className="helper">
                {p.last_check
                  ? `Last response: HTTP ${p.last_check.status}; authentication remains unverified.`
                  : "No authentication check has run."}
              </p>
              <div className="form-actions">
                {p.credential_state === "target_removed" && (
                  <p className="helper">
                    Target removed. This identity is retained for history and
                    cannot be used. Remove it to delete its stored credential.
                  </p>
                )}
                <Button
                  className="button secondary"
                  disabled={busy || p.credential_state === "target_removed"}
                  title={
                    p.credential_state === "target_removed"
                      ? "Target removed; this retained identity can only be removed"
                      : "Edit identity metadata or replace its credential"
                  }
                  onClick={() => {
                    setEditing(p);
                    setTarget(p.target_id);
                    setName(p.name);
                    setRole(p.role);
                    setKind(p.kind);
                    setHeader(p.header_name);
                    setSecret("");
                    setUser("");
                    setShow(false);
                    setNotice("");
                  }}
                >
                  Edit identity
                </Button>
                <Button
                  className="button secondary"
                  disabled={busy || p.expired}
                  onClick={() => {
                    onTest(p);
                    onClose();
                  }}
                  disabledReason={
                    busy
                      ? "An operation is in progress"
                      : "Replace the expired credential first"
                  }
                >
                  Test with reviewed request
                </Button>
                <Button
                  className="button ghost"
                  disabled={busy || p.expired}
                  onClick={() =>
                    void act(async () => {
                      await request("http_auth_expire", {
                        id: p.id,
                        revision: p.revision,
                      });
                      setNotice(
                        "Identity expired; queued approvals are invalidated.",
                      );
                    })
                  }
                  disabledReason={
                    busy
                      ? "An operation is in progress"
                      : "Replace the expired credential first"
                  }
                >
                  Mark expired
                </Button>
                <Button
                  className="button ghost danger"
                  disabled={busy}
                  onClick={() =>
                    void act(async () => {
                      await request("http_auth_remove", {
                        id: p.id,
                        revision: p.revision,
                      });
                      if (editing?.id === p.id) reset();
                      setNotice(
                        "Identity removed; OS credential deletion completed.",
                      );
                    })
                  }
                  disabledReason={"An operation is in progress"}
                >
                  Remove identity
                </Button>
              </div>
            </article>
          ))}
        </section>
        <form
          className="http-identity-form"
          onSubmit={(e) => {
            e.preventDefault();
            void act(async () => {
              const value = await request<AuthProfile>("http_auth_save", {
                id: editing?.id ?? null,
                revision: editing?.revision ?? null,
                input: {
                  project_id: project,
                  target_id: targetId,
                  name,
                  role,
                  kind,
                  header_name: header,
                },
                secret:
                  kind === "anonymous" || !secret
                    ? null
                    : kind === "basic"
                      ? `${user}:${secret}`
                      : secret,
              });
              setNotice(
                value.cleanup_pending
                  ? "Saved without testing; retired credential cleanup is pending. Use reconx auth cleanup to retry."
                  : "Identity saved securely. No authentication request sent.",
              );
              reset();
            });
          }}
        >
          <h3>{editing ? "Edit test identity" : "Add test identity"}</h3>
          <label className="field">
            <span>Associated target</span>
            <select
              aria-label="Identity target"
              value={targetId}
              disabled={busy || !!editing}
              required
              onChange={(e) => setTarget(e.target.value)}
            >
              <option value="">Select owned loopback target</option>
              {(data.targets ?? [])
                .filter((t) => t.project_id === project)
                .map((t) => (
                  <option value={t.id} key={t.id}>
                    {t.name} · {t.value}
                  </option>
                ))}
            </select>
          </label>
          <div className="form-grid">
            <label>
              Name
              <input
                aria-label="Identity name"
                value={name}
                required
                disabled={busy}
                onChange={(e) => setName(e.target.value)}
              />
            </label>
            <label>
              Role
              <input
                aria-label="Identity role"
                value={role}
                required
                disabled={busy}
                onChange={(e) => setRole(e.target.value)}
              />
            </label>
          </div>
          <label className="field">
            <span>Authentication method</span>
            <select
              aria-label="Identity authentication type"
              value={kind}
              disabled={busy}
              onChange={(e) => {
                setKind(e.target.value);
                setSecret("");
                setUser("");
              }}
            >
              {[
                ["anonymous", "Anonymous"],
                ["cookie", "Session cookie"],
                ["bearer", "Bearer token"],
                ["api_key", "API key header"],
                ["basic", "Basic authentication"],
                ["custom_header", "Custom header"],
              ].map(([id, label]) => (
                <option value={id} key={id}>
                  {label}
                </option>
              ))}
            </select>
          </label>
          {["api_key", "custom_header"].includes(kind) && (
            <label className="field">
              <span>Authentication header name</span>
              <input
                aria-label="Identity header name"
                value={header}
                disabled={busy}
                required
                onChange={(e) => setHeader(e.target.value)}
              />
            </label>
          )}
          {kind === "basic" && (
            <label className="field">
              <span>Username (enter again when replacing credentials)</span>
              <input
                aria-label="Identity basic username"
                value={user}
                disabled={busy}
                onChange={(e) => setUser(e.target.value)}
                autoComplete="off"
              />
            </label>
          )}
          {kind !== "anonymous" && (
            <>
              <label className="field">
                <span>
                  {editing
                    ? "Replacement credential (leave empty to retain)"
                    : "Credential"}
                </span>
                <input
                  type={show ? "text" : "password"}
                  aria-label="Identity credential"
                  value={secret}
                  disabled={busy}
                  required={!editing}
                  onChange={(e) => setSecret(e.target.value)}
                  autoComplete="new-password"
                />
              </label>
              <Button
                type="button"
                className="button ghost"
                aria-pressed={show}
                onClick={() => setShow((v) => !v)}
              >
                {show ? "Hide entered credential" : "Show entered credential"}
              </Button>
              <p className="helper">
                Only newly entered credentials can be shown. Cookie format:
                session=token. Token/password/cookie values require at least
                eight characters.
              </p>
            </>
          )}
          <div className="form-actions">
            <Button
              className="button primary"
              disabled={busy || !targetId}
              type="submit"
              disabledReason={
                busy
                  ? "An operation is in progress"
                  : "Select a target before creating an authentication profile"
              }
            >
              Save identity
            </Button>
            {editing && (
              <Button
                className="button secondary"
                type="button"
                disabled={busy}
                onClick={reset}
                disabledReason={"An operation is in progress"}
              >
                Cancel identity edit
              </Button>
            )}
          </div>
        </form>
      </div>
    </Modal>
  );
}
