# Dependency security assessment — 2026-10-18

Actual repository advisories were retrieved with GitHub's authenticated Dependabot
API, using the configured credential helper and a credential-free repository URL.
No Git remote configuration or exposed token was recovered. Export:
`artifacts/dependabot-before.json` (HTTP 200, nine open alerts). The API uses cursor
pagination; the initial rejected page-parameter requests were not counted as scans.

## Eight High alerts: test-only pypdf

Dependency path: `tests/e2e/requirements.txt -> pypdf==6.16.1`.
Despite GitHub's runtime label, pypdf is an independent **test parser**, installed
by the verification workflow. Desktop/CLI PDF generation uses Rust/krilla; pypdf
is not included in their production executables. Tests call `PdfReader`, iterate
pages and extract text from generated local reports. Stream, font and indirect
object parsing can therefore be exercised. The tests do not intentionally use
form appearance generation, attachment enumeration or page-label lookup. This is
an exposure assessment, not justification for retaining a vulnerable parser.

| Alert | Advisory | Affected pin | First patched | Affected operation |
|---|---|---|---|---|
| #2 | [GHSA-qv6h-rv94-w285](https://github.com/advisories/GHSA-qv6h-rv94-w285) | 6.16.1 | 6.17.0 | Roman page-label memory use |
| #3 | [GHSA-5jq2-8x83-x246](https://github.com/advisories/GHSA-5jq2-8x83-x246) | 6.16.1 | 6.18.0 | Indirect-object CPU/memory use |
| #4 | [GHSA-fp3h-c4fm-7vvf](https://github.com/advisories/GHSA-fp3h-c4fm-7vvf) | 6.16.1 | 6.18.1 | ToUnicode stream memory use |
| #5 | [GHSA-g9cg-prrw-2r8q](https://github.com/advisories/GHSA-g9cg-prrw-2r8q) | 6.16.1 | 6.18.1 | Font-data memory use |
| #6 | [GHSA-jw7q-gvrg-4vj3](https://github.com/advisories/GHSA-jw7q-gvrg-4vj3) | 6.16.1 | 6.18.1 | Malformed FlateDecode CPU use |
| #7 | [GHSA-w23x-9jrw-r45c](https://github.com/advisories/GHSA-w23x-9jrw-r45c) | 6.16.1 | 6.19.0 | Alphabetical page-label memory use |
| #8 | [GHSA-php9-fj8v-98fj](https://github.com/advisories/GHSA-php9-fj8v-98fj) | 6.16.1 | 6.19.0 | Appearance-stream CPU use |
| #9 | [GHSA-v247-6f48-mgcj](https://github.com/advisories/GHSA-v247-6f48-mgcj) | 6.16.1 | 6.19.0 | Attachment lookup CPU use |

Resolution: pin **6.19.0**, a compatible 6.x update covering all eight first-patched
versions, with official PyPI wheel SHA-256
`7e5d6e730e7dae87d560a2cee218b852f6498c8be61966f3cd02ead971e48d14`.
No Rust/npm package was upgraded to address these Python advisories.

Provenance: [PyPI release metadata](https://pypi.org/pypi/pypdf/6.19.0/json),
[upstream release](https://github.com/py-pdf/pypdf/releases/tag/6.19.0).
Installed into `artifacts/security-python` with `--only-binary=:all: --require-hashes`;
ordinary user Python packages were not replaced. Actual native output acceptance
with this interpreter passed 6 checkpoints, including independent Unicode PDF
extraction/redaction: `artifacts/outputs-1792322689103/results.json` and
`pdf-inspection.json`; command log: `artifacts/security-pypdf-native.log`.
These exercised the earlier immutable app binaries; final changed-source regression
is recorded separately in TEST_RESULTS.md.

State: **PATCHED_IN_SOURCE and REMOTE_FIXED_VERIFIED**. After the authorized push
of `34421a8`, a fresh authenticated export confirmed alerts #2-#9 as `fixed`, with
fixed_at timestamps 2026-10-18 11:45:53-55 UTC:
`artifacts/dependabot-after-second-check.json`. A separate API contents check
verified that main contains the exact local requirements pin. Alert #1 remains
open. No alert was dismissed or severity changed.

## Medium #1: Linux GTK/glib unsound iterator — unresolved

[GHSA-wrw7-89jp-8q8g](https://github.com/advisories/GHSA-wrw7-89jp-8q8g) /
[RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html):
`glib 0.18.5`, affected `>=0.15.0,<0.20.0`, first patched `0.20.0`.
The upstream issue concerns an incorrectly mutable FFI out-argument in
`VariantStrIter`; optimized use can violate Rust's safety invariants and crash.

Verified dependency path: Linux Desktop -> Tauri 2.12.3 -> GTK 0.18.2 and
WebKit2GTK 2.0.2 -> glib 0.18.5. There are multiple GTK/GIO paths, recorded in
`artifacts/security-glib-tree.txt` (`cargo tree --locked --workspace --target
x86_64-unknown-linux-gnu -i glib`). Tauri's GTK family requires the 0.18 API;
0.20 is not a drop-in semver-compatible replacement. Adding another glib version
does not remove this transitive copy. The official 0.18 source still contains the
affected implementation. No unreviewed fork, fabricated version bump or advisory
ignore was introduced.

Application code has no direct VariantStrIter calls. This does **not** establish
unreachability inside GTK/GIO/Tauri. Windows and Linux Core/CLI tests do not verify
the Linux GUI dependency's safety. Linux Desktop remains unverified and blocked;
this report does not grant a release risk waiver. A compatible upstream fix or a
reviewed, provenance-preserving backport with actual Linux tests is required.

## Other scan results and maintenance risks

`npm audit --json --registry https://registry.npmjs.org` returned 0 findings.
`cargo-audit 0.22.2` reports 0 entries in its **vulnerabilities** list, but also
**one unsound warning** above and six **unmaintained** warnings. Its default exit
code 0 must not be reported as a clean release audit.

| RustSec warning | Dependency |
|---|---|
| RUSTSEC-2024-0370 | proc-macro-error 1.0.4 |
| RUSTSEC-2026-0206 | rustybuzz 0.20.1 |
| RUSTSEC-2026-0192 | ttf-parser 0.25.1 |
| RUSTSEC-2026-0371 | zbus_names 4.3.4 |
| RUSTSEC-2026-0370 | zvariant 5.15.0 |
| RUSTSEC-2026-0372 | zvariant_derive 5.15.0 |

These are maintenance warnings, not six newly established exploitable High
vulnerabilities. They require upstream/path review (font/report and Linux desktop/
credential dependencies) before release. Raw input: `artifacts/security-rust-before.json`.
No broad dependency upgrade or warning suppression was performed.

`scripts/dependency_readiness.py` combines local GitHub/npm/RustSec exports and
lockfile pins. It retains remote alert state independently of source fixes, treats
failed scans as incomplete, and blocks on unsound warnings even when cargo audit
returns 0. Its JSON report is `artifacts/dependency-readiness.json`.
CI now includes the explicit `cargo audit --deny unsound` gate, which is expected
to remain red until the glib issue is resolved. Do not mask it with `--ignore`.

## Previously exposed Git credential

A bounded local scan examined 1,794 artifact text/log/JSON/script files for GitHub
token, private-key and credential-URL patterns without printing matches. No pattern
matches were found in that scope (`artifacts/security-log-scan.json`). Browser
profiles, Git configuration/history, ordinary databases, binary evidence and
dependency directories were excluded. This is not proof that all past copies are
clean or that a previously exposed credential is revoked. No credentials were
recovered, tried, rotated or deleted; revocation remains the user's separate action.
