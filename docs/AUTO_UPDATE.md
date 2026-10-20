# Application update status — 2026-10-19

NOT IMPLEMENTED. Existing packaging configurations and a historical unsigned
Windows NSIS development installer do not implement application auto-update.
Current binaries remain portable development candidates at version 0.1.0.
Linux package/GUI and clean install/upgrade/uninstall gates remain open.

The official Tauri updater plugin, approved endpoint/channel configuration,
signature verification fixture, download/install/restart controls and periodic
checks are still required. No signing key or approved production feed is supplied.
Signed publication is BLOCKED; no public update/release has been published.

Database protection already uses a consistent SQLite backup before an older
supported schema migrates, followed by a transaction. The extension checkpoint
tests real 13→14 upgrades with queued work on Windows and Linux CLI. This does
not establish complete workspace restore or updater/installer recovery acceptance.
