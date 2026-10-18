# Security Policy

ReconX AI is under active development. The current product version is **0.1.0**;
it has not completed v1.0 release acceptance or a comprehensive security audit.
Passing tests and integrity checks do not establish that it is free of vulnerabilities.

This policy covers ReconX AI itself: the Desktop application, CLI, shared Rust
services, credential handling, tool installation, MCP gateway, AI integrations,
local storage and report exports.

## Supported versions

| Version / source | Security maintenance |
| --- | --- |
| Current development branch (0.1.0) | Best-effort fixes on the current branch |
| Older commits, intermediate builds and previous snapshots | No separately maintained security patch branch |

Include the exact commit or build identifier in a report; two development builds
can share the same version number. There is no fixed security update schedule.

## Reporting a vulnerability

Please report security issues privately before publishing technical details.

1. Use GitHub's [Report a vulnerability](https://github.com/dinhvaren/ReconXAI/security/advisories/new)
   form **if private vulnerability reporting is enabled for this repository**.
2. If that form is unavailable, open a public issue titled
   **Request for private security contact**. Include only a request for a private
   reporting channel; do not include vulnerability details or sensitive attachments.
   Wait for a private channel before sharing the report.

Adding this file does not enable private reporting on GitHub. See
[GitHub's private reporting documentation](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/report-privately).
No separate security email address or encrypted contact channel is currently
documented here.

A useful private report includes:

- The affected commit/build, operating system and relevant component.
- A description of the issue, its impact and the trust boundary it crosses.
- Minimal reproduction steps using a local fixture or an authorized test environment.
- Required configuration, permissions and user interactions.
- Sanitized logs, screenshots or a small synthetic sample, if needed.
- A suggested mitigation, if known.

Do not send real API keys, authentication cookies, private keys, credential-bearing
Git URLs, customer data or proprietary binaries. Replace secrets with synthetic
values and remove sensitive local paths. If a credential has been exposed, revoke
or rotate it through its issuer; do not paste it into the report.

Reports are handled on a best-effort basis, without guaranteed response or fix
deadlines. Maintainers may request more information, assess the impact, and
coordinate a fix and disclosure. Please keep discussion in the private channel
and coordinate publication of reproducible details.

## Relevant security issues

Examples include credential disclosure, command injection, unsafe path or archive
handling, exploitable file parsing, installer integrity bypass, unauthorized data
transmission, or bypasses of scope, approval, IPC or MCP invocation policies.
AI or MCP content that causes ReconX AI to cross these boundaries is also relevant.

Ordinary crashes, missing features and UI defects without a security impact can
use public issues after removing sensitive data. A vulnerability in a third-party
tool, server or model should also be reported to its upstream maintainer; report
it here when ReconX AI's integration introduces or amplifies the exposure. If you
are unsure whether an issue has security impact, request private contact first.

## Current trust boundaries and limitations

- **Local account trust:** configured MCP stdio servers and external tools run as
  user-account processes. They are not OS-sandboxed. Invocation policies and a
  restricted child environment do not prevent a malicious executable from using
  account-level filesystem or credential APIs.
- **Stored data:** API keys and private gateway headers use OS credential storage,
  with no plaintext fallback. SQLite and evidence artifacts are not encrypted by
  the application. Protect the workspace and backups with appropriate filesystem
  permissions and disk encryption where needed.
- **AI and network destinations:** providers, gateways and explicit proxies are
  external trust decisions. Review the destination and content before transmission.
  Selected-artifact AI review requires separate consent and currently rejects
  configured proxies. A saved provider or catalog preset does not prove model
  access, service availability or privacy guarantees.
- **Evidence and exports:** SHA-256 verifies retained bytes, not the truth of a
  finding. AI output remains unverified interpretation. Report redaction is
  best-effort; review previews and attachments before sharing. Raw evidence can
  contain credentials and other sensitive material.
- **Platform coverage:** Windows Desktop/CLI and Linux shared Core/CLI have tested
  workflows. Linux Desktop, Linux credential-store runtime, installers and ARM64
  are not independently verified. Implemented workflows do not establish complete
  platform or security-tool support.

These limitations describe the current protection model; they do not make new
security weaknesses unreportable. Detailed controls and verification evidence are
in [Security model](docs/SECURITY_MODEL.md),
[AI Provider Manager](docs/AI_PROVIDER_MANAGER.md) and
[Test results](docs/TEST_RESULTS.md).

## Responsible testing

Use isolated local workspaces, synthetic credentials, benign files and systems you
own or are explicitly authorized to test. Do not access other users' data, disrupt
services, or scan public infrastructure to demonstrate an application defect.
Project registration and a selected research mode do not create authorization to
test a third-party target.
