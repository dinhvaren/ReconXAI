// Real, benign MCP fixture. No shell, filesystem tool or external requests.
import { createInterface } from "node:readline";
import { createServer } from "node:http";
const schema = process.argv.includes("--bad-schema")
  ? { type: "invalid-type" }
  : {
      type: "object",
      properties: {
        text: { type: "string", maxLength: 500 },
        delay_ms: { type: "integer", minimum: 0, maximum: 10000 },
      },
      required: ["text"],
      additionalProperties: false,
    };
const tools = [
  {
    name: "echo",
    description: "Return supplied text, without external actions",
    inputSchema: schema,
    annotations: {
      readOnlyHint: true,
      destructiveHint: false,
      openWorldHint: false,
    },
  },
  {
    name: "denied",
    description: "Denied fixture tool",
    inputSchema: { type: "object" },
  },
];
async function response(m) {
  if (m.id === undefined) return null;
  let result;
  switch (m.method) {
    case "initialize":
      result = {
        protocolVersion: "2025-11-25",
        capabilities: { tools: {} },
        serverInfo: { name: "reconx-local-fixture", version: "1.0.0" },
      };
      break;
    case "ping":
      result = {};
      break;
    case "tools/list":
      if (process.argv.includes("--echo-server-secret"))
        tools[0].description = process.env.MCP_TEST_TOKEN;
      result = { tools };
      break;
    case "tools/call":
      if (m.params.name !== "echo")
        return {
          jsonrpc: "2.0",
          id: m.id,
          error: { code: -32602, message: "No such approved fixture tool" },
        };
      if (m.params.arguments.delay_ms)
        await new Promise((r) => setTimeout(r, m.params.arguments.delay_ms));
      result = {
        content: [
          {
            type: "text",
            text: process.argv.includes("--echo-server-secret")
              ? process.env.MCP_TEST_TOKEN
              : process.argv.includes("--large-output")
                ? "x".repeat(300000)
                : m.params.arguments.text,
          },
        ],
        structuredContent: {
          echo: m.params.arguments.text,
          host_secret_inherited: !!process.env.RECONX_TEST_PROVIDER_SECRET,
        },
        isError: false,
      };
      break;
    default:
      return {
        jsonrpc: "2.0",
        id: m.id,
        error: { code: -32601, message: "Method not found" },
      };
  }
  return { jsonrpc: "2.0", id: m.id, result };
}
if (process.argv.includes("--http")) {
  const server = createServer(async (req, res) => {
    if (req.method === "DELETE") {
      res.writeHead(200);
      res.end();
      return;
    }
    if (req.method !== "POST") {
      res.writeHead(405);
      res.end();
      return;
    }
    if (
      process.argv.includes("--auth") &&
      req.headers.authorization !== "Bearer reconx-fixture-token"
    ) {
      res.writeHead(401);
      res.end();
      return;
    }
    try {
      let b = "";
      for await (const chunk of req) {
        b += chunk;
        if (b.length > 262144) throw Error("input limit");
      }
      const result = await response(JSON.parse(b));
      if (!result) {
        res.writeHead(202);
        res.end();
      } else {
        res.writeHead(200, { "Content-Type": "application/json" });
        res.end(JSON.stringify(result));
      }
    } catch {
      res.writeHead(400);
      res.end();
    }
  });
  server.listen(Number(process.env.RECONX_FIXTURE_PORT ?? 0), "127.0.0.1", () =>
    process.stdout.write(
      JSON.stringify({ port: server.address().port }) + "\n",
    ),
  );
} else {
  const lines = createInterface({ input: process.stdin });
  lines.on("line", async (line) => {
    try {
      const r = await response(JSON.parse(line));
      if (r) process.stdout.write(JSON.stringify(r) + "\n");
    } catch {
      process.exitCode = 1;
      lines.close();
    }
  });
}
