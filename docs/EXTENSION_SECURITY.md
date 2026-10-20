# Extension security boundaries

Only compiled reviewed installer adapters execute. Registry descriptions,
readmes, entrypoints and package arguments never become shell commands. No native
dynamic libraries are loaded. The npm adapter directly downloads HTTPS tarballs
from pinned official npm sources and verifies SHA-512 for every package; the
managed Node artifact and executable have pinned SHA-256. No package lifecycle
scripts are executed. Signature/GPG validation of the Node release is not claimed.

Download size/time, expanded size, file counts/depth and receipt sizes are bounded.
Archive traversal, absolute/device paths, links and duplicate files are rejected;
unused Node archive entries are skipped. Installation uses owned temporary staging
directories, checked free space, atomic renames and verified receipts. Cancellation
is cooperative, not an operating-system resource sandbox. A failed operation must
not replace the previous active MCP configuration.

Generated MCP configuration contains absolute executable/arguments, owned working
directory, no secret values, and an initially empty tool allowlist. Health checks
only initialize MCP and discover/validate real tool schemas. Tool permissions need
separate explicit approval and read-only, closed-world annotations; exact invocation
input needs another approval. Existing MCP output limits/audit/history apply.
Tool annotations are claims by third-party code and do not sandbox that code.

The process environment is cleared using the existing MCP transport. AI credential
store access is not passed to the server. Package/runtime integrity is checked
before managed connections/invocation; modified configurations are preserved and
reported as conflicts. Disable cancels bound MCP jobs and disconnects the local
session. Uninstall deletes verified owned package files and generated registration,
retaining shared runtimes, cached prior versions and research evidence/history.

**No OS sandbox is implemented for these npm servers.** Approved server code runs
at the user's privileges and can access resources available to that user. Network
origin restrictions apply to the Rust downloader, not arbitrary Node code. Do not
interpret tool allowlists as filesystem/network isolation. Stronger Windows/Linux
process isolation remains a release backlog item. Same-user filesystem races and
unbounded native third-party code cannot be eliminated by archive/hash validation.
