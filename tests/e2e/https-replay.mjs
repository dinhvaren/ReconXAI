// Local negative TLS acceptance. The generated one-day test key is never stored in source.
import { createServer } from "node:https";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { mkdir, readFile, writeFile, unlink } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
import assert from "node:assert/strict";
const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../..",
);
const artifact = path.join(root, "artifacts", `https-replay-${Date.now()}`);
await mkdir(artifact, { recursive: true });
const cli =
  process.env.RECONX_CLI_EXE ?? path.join(root, "target/debug/reconx.exe");
const openssl =
  process.env.RECONX_TEST_OPENSSL ??
  (process.platform === "win32"
    ? "C:/Program Files/Git/usr/bin/openssl.exe"
    : "openssl");
const env = { ...process.env, RECONX_DATA_DIR: artifact };
const exec = promisify(execFile);
const key = path.join(artifact, "ephemeral-test-key.pem"),
  cert = path.join(artifact, "ephemeral-test-cert.pem");
let server,
  connections = 0,
  requests = 0;
const sockets = new Set();
async function command(args) {
  const out = await exec(cli, ["--json", ...args], { env, windowsHide: true });
  return out.stdout.trim() ? JSON.parse(out.stdout) : null;
}
try {
  await exec(
    openssl,
    [
      "req",
      "-x509",
      "-newkey",
      "rsa:2048",
      "-keyout",
      key,
      "-out",
      cert,
      "-sha256",
      "-days",
      "1",
      "-nodes",
      "-subj",
      "/CN=localhost",
      "-addext",
      "subjectAltName=IP:127.0.0.1,DNS:localhost",
    ],
    { windowsHide: true },
  );
  server = createServer(
    { key: await readFile(key), cert: await readFile(cert) },
    (_req, res) => {
      requests++;
      res.end("Untrusted test certificate must not reach HTTP");
    },
  );
  server.on("tlsClientError", () => {});
  server.on("connection", (socket) => {
    connections++;
    sockets.add(socket);
    socket.on("close", () => sockets.delete(socket));
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const project = await command(["project", "create", "Negative TLS fixture"]);
  const url = `https://127.0.0.1:${server.address().port}/`;
  const target = await command([
    "target",
    "add",
    "--project",
    project.id,
    "--type",
    "rest_api",
    "--value",
    url,
    "--authorized",
    "--mode",
    "lab_research",
  ]);
  const settings = await command(["settings", "show"]);
  settings.security_mode = "lab_research";
  const settingsFile = path.join(artifact, "settings.json");
  await writeFile(settingsFile, JSON.stringify(settings));
  await command(["settings", "apply", settingsFile]);
  const requestFile = path.join(artifact, "request.json");
  await writeFile(
    requestFile,
    JSON.stringify({
      project_id: project.id,
      target_id: target.id,
      source_exchange_id: null,
      title: "Reject untrusted local certificate",
      method: "GET",
      url,
      headers: [],
      content_type: "",
      body: "",
      auth_profile_id: null,
      timeout_ms: 5000,
      follow_redirects: false,
      proxy: null,
    }),
  );
  const draft = await command(["http", "save", requestFile]);
  const plan = await command(["http", "preview", draft.id]);
  const queued = await command([
    "http",
    "replay",
    draft.id,
    "--fingerprint",
    plan.fingerprint,
    "--approve",
  ]);
  await command(["jobs", "run"]);
  const job = (await command(["http", "history", draft.id])).find(
    (j) => j.id === queued.id,
  );
  assert.equal(job.status, "failed");
  assert.equal(job.error_code, "HTTP_CONNECT");
  assert.equal(job.result, null);
  assert(connections >= 1);
  assert.equal(requests, 0);
  const record = {
    status: "passed",
    scope:
      "Negative TLS trust only: self-signed local certificate, no trust store changes or external traffic",
    cli,
    job_id: job.id,
    error_code: job.error_code,
    tcp_connections: connections,
    http_requests: requests,
    private_test_key_removed_on_exit: true,
  };
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify(record, null, 2),
  );
  console.log(JSON.stringify({ artifact, ...record }, null, 2));
} catch (error) {
  await writeFile(
    path.join(artifact, "results.json"),
    JSON.stringify({ status: "failed", error: String(error) }, null, 2),
  );
  throw error;
} finally {
  for (const socket of sockets) socket.destroy();
  if (server) await new Promise((r) => server.close(r));
  for (const file of [key, cert]) {
    assert(!path.relative(artifact, file).startsWith(".."));
    await unlink(file).catch((e) => {
      if (e.code !== "ENOENT") throw e;
    });
  }
}
