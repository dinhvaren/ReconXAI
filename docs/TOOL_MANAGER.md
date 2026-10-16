# Tool Manager

The shared `reconx-tools` manager installs pinned official portable executables.
Desktop and CLI use the directory beside their shared database, plus SQLite
installation snapshots and the existing audit log. Ordinary PATH executables are
not silently adopted or executed. No elevated package manager runs automatically.

| Adapter | Verified release | Operations | Platform verification |
| --- | --- | --- | --- |
| ripgrep | 15.2.0 | Existing verified install/version/removal | Windows; Linux runtime unverified |
| YARA-X | 1.21.0 | Verified install/version/removal; fixed offline file signatures through Job Engine | Windows native install/signature/repair/profile acceptance passed; Linux runtime unverified |

Both archive and extracted executable SHA-256 are pinned in `manifests/tools`.
Only the named regular archive entry is copied, with a 32 MiB bound. Sources are
restricted to official GitHub release downloads and approved GitHub asset hosts.
Installation publishes a verified executable and receipt; health runs the fixed
`--version` argument. Uninstall removes named managed files only, preserving
unexpected user files. Repair explicitly removes managed files and reinstalls.
The UI shows actual download bytes and supports cancellation before publication.

YARA-X signatures use the existing jobs/leases/cancellation machinery, one verified
managed sample, one process/thread, fixed embedded rules and bounded output. Rules
identify PE/ELF headers and the benign acceptance marker; they do **not** assert
vulnerabilities, execute samples or scan network targets. No user shell arguments
or third-party rule includes are accepted. Output retains hashes/rule identifiers,
with temporary sample paths removed. CLI result records are identical to Desktop.

## CLI

```
reconx tools list
reconx tools host
reconx tools search yara
reconx tools info yara-x
reconx tools plan yara-x
reconx tools install yara-x
reconx tools verify yara-x
reconx tools update yara-x
reconx tools remove yara-x
reconx tools profiles
reconx tools install-profile reverse
reconx tools install-profile reverse --approve
reconx jobs enqueue --kind yara_signatures --file <imported-file-id>
reconx jobs run
reconx jobs result <job-id>
```

Use global `--json` and `--database` as with existing commands. `update` checks the
latest **reviewed, pinned manifest**, not the latest unreviewed upstream release.
The `reverse` profile contains ripgrep/YARA-X; `core` contains ripgrep. Dependency
sets are empty for these portable adapters. Other profiles/catalog tools are
explicitly pending and cannot be installed. Package manager execution, WSL execution,
arbitrary external executable configuration, general dependency DAGs, automatic
upstream upgrades and multi-version rollback remain incomplete. Download byte logs
are live; audit history retains lifecycle outcomes, not every received chunk.

Sources: [official installation guide](https://virustotal.github.io/yara-x/docs/intro/installation/),
[reviewed release](https://github.com/VirusTotal/yara-x/releases/tag/v1.21.0).
`manifests/tools/SOURCES.md` records archive/executable verification provenance.
