# Verified tool source records

YARA-X 1.21.0 metadata was read from the official GitHub API:
https://api.github.com/repos/VirusTotal/yara-x/releases/tags/v1.21.0

Both official archives were downloaded during Windows development. Each archive
SHA-256 was compared with the release asset's published `digest`; the single
executable entry was then read and independently hashed. Pinned values are stored
in yara-x.json. No downloaded binaries belong in Git. This verifies byte integrity
against upstream HTTPS/GitHub metadata, not an independent detached publisher
signature or Linux runtime compatibility.

- Windows zip: archive `0e2fc4d2f64df3eaa22129ad5bd074c968a5d80766ddec6183b73175e5c9da25`;
  `yr.exe` `883c56964e0ffbe25eb0daf5b443914bea9db45d8dc4b6be7abbe59ff6797557`.
- Linux tar.gz: archive `01585181e8979e36ac10ab123fcf8921cf8ba879bf942916413e091e18d64852`;
  `yr` `a4b6fec172c813cdfc099a4a392939cfc2c555e9e7745fb4b5ed369ab1edf93b`.

Official release: https://github.com/VirusTotal/yara-x/releases/tag/v1.21.0
