# Packaging

`scripts/build.ps1` produces portable Windows x64 release binaries and their SHA-256 digests. Run the desktop executable directly; CLI commands use the same application data directory.

`scripts/build.ps1 -Installer` additionally requests a per-user NSIS installer using the official Tauri bundler. This does not install ReconX AI or privileged packages. It may download bundler tooling to the Tauri cache. Installer generation and installation are separate acceptance tests; do not infer install success from a binary build.

Linux CI builds the desktop and CLI on Ubuntu with declared GTK/WebKit/OpenSSL development dependencies, and creates a Debian package through the Tauri CLI. Until CI runs successfully, Linux builds and packages are unverified.

Artifacts are unsigned. A production release needs a trusted signing certificate, update signing strategy, installer validation and a release audit. CLI PATH registration and updater distribution are not implemented.

Release bundles do not contain development-only test databases, provider credentials, local screenshots or node_modules.
