//! Bounded, read-only analysis of user-selected local bytes. Never executes samples.
use object::{Object, ObjectSection, ObjectSegment};
use reconx_security::{AppError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Cursor;

pub const MAX_FILE_BYTES: usize = 64 * 1024 * 1024;
const MAX_ROWS: usize = 4096;
const MAX_STRINGS: usize = 4000;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub name: String,
    pub address: u64,
    pub size: u64,
    pub file_offset: Option<u64>,
    pub entropy: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryString {
    pub offset: u64,
    pub encoding: String,
    pub text: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryAnalysis {
    pub format: String,
    pub architecture: Option<String>,
    pub size: u64,
    pub sha256: String,
    pub entry_point: Option<u64>,
    pub little_endian: Option<bool>,
    pub entropy: f64,
    pub sections: Vec<Section>,
    pub segments: Vec<Section>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub package_entries: Vec<String>,
    pub strings: Vec<BinaryString>,
    pub hex_preview: String,
    pub warnings: Vec<String>,
}
pub fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub fn entropy(bytes: &[u8]) -> f64 {
    if bytes.is_empty() {
        return 0.0;
    }
    let mut counts = [0usize; 256];
    for b in bytes {
        counts[*b as usize] += 1;
    }
    counts
        .iter()
        .filter(|&&n| n != 0)
        .map(|&n| {
            let p = n as f64 / bytes.len() as f64;
            -p * p.log2()
        })
        .sum()
}
fn safe_label(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .take(256)
        .collect()
}
fn symbol_name(name: object::NameOrOrdinal<&[u8]>) -> String {
    match name {
        object::NameOrOrdinal::Name(n) => safe_label(&String::from_utf8_lossy(n)),
        object::NameOrOrdinal::Ordinal(n) => format!("ordinal {n}"),
    }
}
fn extract_strings(
    bytes: &[u8],
    update: &mut dyn FnMut(u8, &str) -> Result<()>,
) -> Result<Vec<BinaryString>> {
    let mut rows = Vec::new();
    let mut start = 0;
    let mut next_check = 0;
    while start < bytes.len() && rows.len() < MAX_STRINGS / 2 {
        if start >= next_check {
            update(
                (30 + start * 20 / bytes.len().max(1)) as u8,
                "Extracting bounded ASCII strings",
            )?;
            next_check = start + 1024 * 1024;
        }
        if (32..=126).contains(&bytes[start]) {
            let mut end = start;
            while end < bytes.len() && (32..=126).contains(&bytes[end]) {
                end += 1;
                if end >= next_check {
                    update(
                        (30 + end * 20 / bytes.len().max(1)) as u8,
                        "Extracting bounded ASCII strings",
                    )?;
                    next_check = end + 1024 * 1024;
                }
            }
            if end - start >= 4 {
                rows.push(BinaryString {
                    offset: start as u64,
                    encoding: "ASCII".into(),
                    text: String::from_utf8_lossy(&bytes[start..end.min(start + 256)]).into_owned(),
                });
            }
            start = end;
        } else {
            start += 1;
        }
    }
    for alignment in 0..2 {
        let mut start = alignment;
        let mut next_check = 0;
        while start + 1 < bytes.len() && rows.len() < MAX_STRINGS {
            if start >= next_check {
                update(
                    (50 + alignment * 10 + start * 10 / bytes.len().max(1)) as u8,
                    "Extracting bounded UTF-16 LE strings",
                )?;
                next_check = start + 1024 * 1024;
            }
            // UTF-16 LE strings: anchor on a printable ASCII code unit, then accept
            // valid non-control BMP characters as well. No surrogate fabrication.
            if (32..=126).contains(&bytes[start])
                && bytes[start + 1] == 0
                && (start == 0 || !(32..=126).contains(&bytes[start - 1]))
            {
                let mut units = Vec::new();
                let mut end = start;
                while end + 1 < bytes.len() && units.len() < 256 {
                    let unit = u16::from_le_bytes([bytes[end], bytes[end + 1]]);
                    if char::from_u32(unit as u32).is_none_or(|c| c.is_control()) {
                        break;
                    }
                    units.push(unit);
                    end += 2;
                }
                if units.len() >= 4 {
                    rows.push(BinaryString {
                        offset: start as u64,
                        encoding: "UTF-16 LE".into(),
                        text: String::from_utf16_lossy(&units),
                    });
                }
                start = end.max(start + 2);
            } else {
                start += 2;
            }
        }
    }
    rows.sort_by_key(|s| s.offset);
    Ok(rows)
}
pub fn inspect(bytes: &[u8]) -> Result<BinaryAnalysis> {
    inspect_with_progress(bytes, |_, _| Ok(()))
}
pub fn inspect_with_progress(
    bytes: &[u8],
    mut update: impl FnMut(u8, &str) -> Result<()>,
) -> Result<BinaryAnalysis> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(AppError::new("INPUT_LIMIT", "Files are limited to 64 MiB"));
    }
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| parse(bytes, &mut update)))
        .map_err(|_| AppError::new("MALFORMED_BINARY", "Parser rejected malformed input"))?
}
fn parse(bytes: &[u8], update: &mut dyn FnMut(u8, &str) -> Result<()>) -> Result<BinaryAnalysis> {
    let mut sha = Sha256::new();
    let mut counts = [0usize; 256];
    update(0, "Hashing local sample")?;
    for (i, chunk) in bytes.chunks(1024 * 1024).enumerate() {
        sha.update(chunk);
        update(
            (i * 1024 * 1024 * 15 / bytes.len().max(1)) as u8,
            "Hashing local sample",
        )?;
    }
    let sha256 = sha.finalize().iter().map(|b| format!("{b:02x}")).collect();
    for (i, chunk) in bytes.chunks(1024 * 1024).enumerate() {
        for b in chunk {
            counts[*b as usize] += 1;
        }
        update(
            (15 + i * 1024 * 1024 * 15 / bytes.len().max(1)) as u8,
            "Measuring file entropy",
        )?;
    }
    let file_entropy = counts
        .iter()
        .filter(|&&n| n != 0)
        .map(|&n| {
            let p = n as f64 / bytes.len() as f64;
            -p * p.log2()
        })
        .sum();
    let strings = extract_strings(bytes, update)?;
    update(
        70,
        "Parsing content signatures, sections, imports and exports",
    )?;
    let mut result = BinaryAnalysis {
        format: "Raw binary".into(),
        architecture: None,
        size: bytes.len() as u64,
        sha256,
        entry_point: None,
        little_endian: None,
        entropy: file_entropy,
        sections: vec![],
        segments: vec![],
        imports: vec![],
        exports: vec![],
        package_entries: vec![],
        strings,
        hex_preview: bytes
            .iter()
            .take(512)
            .enumerate()
            .map(|(i, b)| {
                format!(
                    "{}{:02x}{}",
                    if i % 16 == 0 {
                        format!("{i:08x}  ")
                    } else {
                        String::new()
                    },
                    b,
                    if i % 16 == 15 { "\n" } else { " " }
                )
            })
            .collect(),
        warnings: vec![],
    };
    if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") {
        // Bound central-directory cardinality before the ZIP reader allocates its
        // entry index. ZIP64 is deliberately unsupported in this bounded slice.
        let eocd = bytes
            .windows(4)
            .enumerate()
            .rev()
            .take(65557)
            .find(|(i, b)| *b == b"PK\x05\x06" && *i + 22 <= bytes.len())
            .map(|(i, _)| i)
            .ok_or_else(|| {
                AppError::new("MALFORMED_BINARY", "ZIP end-of-directory record is missing")
            })?;
        let entries = u16::from_le_bytes([bytes[eocd + 10], bytes[eocd + 11]]) as usize;
        let comment = u16::from_le_bytes([bytes[eocd + 20], bytes[eocd + 21]]) as usize;
        if eocd + 22 + comment != bytes.len() || bytes[eocd + 4..eocd + 8] != [0, 0, 0, 0] {
            return Err(AppError::new(
                "MALFORMED_BINARY",
                "Split archives and invalid end-of-directory records are unsupported",
            ));
        }
        if entries > MAX_ROWS {
            return Err(AppError::new(
                "INPUT_LIMIT",
                "ZIP64 and packages with more than 4096 entries are unsupported",
            ));
        }
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
            .map_err(|_| AppError::new("MALFORMED_BINARY", "Invalid ZIP package directory"))?;
        if archive.len() > MAX_ROWS {
            return Err(AppError::new(
                "INPUT_LIMIT",
                "Package has more than 4096 entries",
            ));
        }
        for i in 0..archive.len() {
            if i % 32 == 0 {
                update(80, "Inspecting bounded package directory")?;
            }
            let entry = archive
                .by_index_raw(i)
                .map_err(|_| AppError::new("MALFORMED_BINARY", "Invalid package entry"))?;
            result.package_entries.push(safe_label(
                &entry
                    .name()
                    .map_err(|_| AppError::new("MALFORMED_BINARY", "Invalid package entry name"))?,
            ));
        }
        result.format = if result
            .package_entries
            .iter()
            .any(|e| e == "AndroidManifest.xml")
        {
            "Android APK"
        } else {
            "ZIP package"
        }
        .into();
        result
            .warnings
            .push("Package directory only; contents are never extracted or executed".into());
    } else {
        match object::File::parse(bytes) {
            Ok(file) => {
                result.format = format!("{:?}", file.format());
                result.architecture = Some(format!("{:?}", file.architecture()));
                result.entry_point = Some(file.entry());
                result.little_endian = Some(file.is_little_endian());
                let mut section_entropy_budget = MAX_FILE_BYTES;
                let mut skipped_entropy = false;
                for section in file.sections().take(MAX_ROWS + 1) {
                    if result.sections.len().is_multiple_of(32) {
                        update(80, "Inspecting binary sections")?;
                    }
                    if result.sections.len() == MAX_ROWS {
                        return Err(AppError::new("INPUT_LIMIT", "Too many sections"));
                    }
                    let section_entropy = section.data().ok().and_then(|data| {
                        if data.len() <= section_entropy_budget {
                            section_entropy_budget -= data.len();
                            Some(entropy(data))
                        } else {
                            skipped_entropy = true;
                            None
                        }
                    });
                    result.sections.push(Section {
                        name: safe_label(section.name().unwrap_or("<invalid name>")),
                        address: section.address(),
                        size: section.size(),
                        file_offset: section.file_range().map(|r| r.0),
                        entropy: section_entropy,
                    });
                }
                for segment in file.segments().take(MAX_ROWS) {
                    result.segments.push(Section {
                        name: segment
                            .name()
                            .ok()
                            .flatten()
                            .map(safe_label)
                            .unwrap_or_default(),
                        address: segment.address(),
                        size: segment.size(),
                        file_offset: Some(segment.file_range().0),
                        entropy: None,
                    });
                }
                if skipped_entropy {
                    result.warnings.push("Section entropy byte budget reached; overlapping section data is not rescanned indefinitely".into());
                }
                match file.imports() {
                    Ok(imports) => {
                        for import in imports.take(MAX_ROWS) {
                            if result.imports.len().is_multiple_of(64) {
                                update(90, "Inspecting import table")?;
                            }
                            match import {
                                Ok(i) => result.imports.push(format!(
                                    "{}!{}",
                                    safe_label(&String::from_utf8_lossy(i.library())),
                                    symbol_name(i.name())
                                )),
                                Err(_) => {
                                    result
                                        .warnings
                                        .push("Some imports could not be parsed".into());
                                    break;
                                }
                            }
                        }
                    }
                    Err(_) => result.warnings.push("Import table unavailable".into()),
                }
                match file.exports() {
                    Ok(exports) => {
                        for export in exports.take(MAX_ROWS) {
                            if result.exports.len().is_multiple_of(64) {
                                update(95, "Inspecting export table")?;
                            }
                            match export {
                                Ok(e) => result.exports.push(format!(
                                    "{} {}",
                                    symbol_name(e.name()),
                                    safe_label(&format!("{:?}", e.target()))
                                )),
                                Err(_) => {
                                    result
                                        .warnings
                                        .push("Some exports could not be parsed".into());
                                    break;
                                }
                            }
                        }
                    }
                    Err(_) => result.warnings.push("Export table unavailable".into()),
                }
            }
            Err(_) => {
                if bytes.starts_with(b"MZ")
                    || bytes.starts_with(b"\x7fELF")
                    || bytes.starts_with(&[0xcf, 0xfa, 0xed, 0xfe])
                    || bytes.starts_with(&[0xfe, 0xed, 0xfa, 0xcf])
                {
                    return Err(AppError::new(
                        "MALFORMED_BINARY",
                        "Recognized executable signature with invalid or unsupported structure",
                    ));
                }
                result.warnings.push(
                    "Unrecognized format; only hash, entropy, hex and strings are available".into(),
                );
            }
        }
    }
    if result.strings.len() == MAX_STRINGS {
        result.warnings.push("String output reached the 4000-row limit; individual strings are capped at 256 characters".into());
    }
    update(100, "Static parser completed")?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_strings_and_entropy() {
        let data = b"\0hello fixture\0w\0i\0d\0e\0\0\0";
        let a = inspect(data).unwrap();
        assert!(a.strings.iter().any(|s| s.text == "hello fixture"));
        assert!(
            a.strings
                .iter()
                .any(|s| s.text.starts_with("wide") && s.encoding == "UTF-16 LE")
        );
        assert_eq!(entropy(&[1, 1]), 0.0);
        assert_eq!(entropy(&[1, 2]), 1.0);
    }
    #[test]
    fn malformed_headers_are_errors_not_fake_raw_analysis() {
        for b in [
            b"MZ".as_slice(),
            b"\x7fELF".as_slice(),
            b"PK\x03\x04".as_slice(),
        ] {
            assert_eq!(inspect(b).unwrap_err().code, "MALFORMED_BINARY");
        }
    }
    #[test]
    fn output_is_bounded() {
        let data = b"abcd\0".repeat(5000);
        assert!(inspect(&data).unwrap().strings.len() <= MAX_STRINGS);
    }
    #[test]
    fn elf_and_macho_are_identified_from_bytes_not_extensions() {
        let mut elf = vec![0u8; 64];
        elf[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
        elf[16..18].copy_from_slice(&2u16.to_le_bytes());
        elf[18..20].copy_from_slice(&62u16.to_le_bytes());
        elf[20..24].copy_from_slice(&1u32.to_le_bytes());
        elf[52..54].copy_from_slice(&64u16.to_le_bytes());
        elf[54..56].copy_from_slice(&56u16.to_le_bytes());
        elf[58..60].copy_from_slice(&64u16.to_le_bytes());
        let a = inspect(&elf).unwrap();
        assert_eq!(a.format, "Elf");
        assert_eq!(a.architecture.as_deref(), Some("X86_64"));
        let mut macho = vec![0u8; 32];
        macho[..4].copy_from_slice(&0xfeedfacfu32.to_le_bytes());
        macho[4..8].copy_from_slice(&0x01000007u32.to_le_bytes());
        macho[8..12].copy_from_slice(&3u32.to_le_bytes());
        macho[12..16].copy_from_slice(&2u32.to_le_bytes());
        let b = inspect(&macho).unwrap();
        assert_eq!(b.format, "MachO");
        assert_eq!(b.architecture.as_deref(), Some("X86_64"));
    }
    #[test]
    fn apk_uses_package_contents_and_never_extracts_entries() {
        use std::io::Write;
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file(
                "AndroidManifest.xml",
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored),
            )
            .unwrap();
        writer.write_all(b"benign manifest fixture").unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        let a = inspect(&bytes).unwrap();
        assert_eq!(a.format, "Android APK");
        assert_eq!(a.package_entries, vec!["AndroidManifest.xml"]);
    }
}
