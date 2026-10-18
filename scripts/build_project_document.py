"""Build an editable, all-black Times New Roman project manual locally.

No network, credentials, user databases, external templates or cloud conversion.
Only an explicitly supplied CLI executable is run, with --help (never a command).
Refuses to replace an existing output document.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import io
import json
from pathlib import Path
import re
import subprocess
import unicodedata
import zipfile

from docx import Document
from docx.enum.style import WD_STYLE_TYPE
from docx.enum.table import WD_TABLE_ALIGNMENT
from docx.enum.text import WD_ALIGN_PARAGRAPH
from docx.oxml import OxmlElement
from docx.oxml.ns import qn
from docx.shared import Cm, Pt, RGBColor
from lxml import etree

FONT = "Times New Roman"
ROOT = Path(__file__).resolve().parents[1]


def font(properties, size=None):
    fonts = properties.find(qn("w:rFonts"))
    if fonts is None:
        fonts = OxmlElement("w:rFonts")
        properties.insert(0, fonts)
    for name in ("ascii", "hAnsi", "eastAsia", "cs"):
        fonts.set(qn("w:" + name), FONT)
    for name in ("asciiTheme", "hAnsiTheme", "eastAsiaTheme", "cstheme", "csTheme"):
        fonts.attrib.pop(qn("w:" + name), None)
    color = properties.find(qn("w:color"))
    if color is None:
        color = OxmlElement("w:color")
        properties.append(color)
    color.attrib.clear()
    color.set(qn("w:val"), "000000")
    if size is not None:
        for tag in ("sz", "szCs"):
            element = properties.find(qn("w:" + tag))
            if element is None:
                element = OxmlElement("w:" + tag)
                properties.append(element)
            element.set(qn("w:val"), str(round(size * 2)))


def text_run(paragraph, value, bold=False):
    run = paragraph.add_run(unicodedata.normalize("NFC", value))
    run.bold = bold
    font(run._r.get_or_add_rPr())
    return run


def add_text(paragraph, value):
    # Formatting is intentionally restrained; code also uses the requested font.
    value = re.sub(r"\[([^]]+)\]\(([^)]+)\)", r"\1 (\2)", value)
    for part in re.split(r"(\*\*[^*]+\*\*|`[^`]+`)", value):
        if part.startswith("**") and part.endswith("**"):
            text_run(paragraph, part[2:-2], bold=True)
        elif part.startswith("`") and part.endswith("`"):
            text_run(paragraph, part[1:-1])
        else:
            text_run(paragraph, part)


def add_table(document, headers, rows):
    table = document.add_table(rows=1, cols=len(headers))
    table.alignment = WD_TABLE_ALIGNMENT.CENTER
    table.style = "Table Grid"
    table.autofit = False
    for cell in table.rows[0].cells:
        shade = OxmlElement("w:shd")
        shade.set(qn("w:fill"), "F0F0F0")
        cell._tc.get_or_add_tcPr().append(shade)
    repeat = OxmlElement("w:tblHeader")
    table.rows[0]._tr.get_or_add_trPr().append(repeat)
    for index, row in enumerate([headers, *rows]):
        cells = table.rows[0].cells if index == 0 else table.add_row().cells
        prevent_split = OxmlElement("w:cantSplit")
        cells[0]._tc.getparent().get_or_add_trPr().append(prevent_split)
        for cell, value in zip(cells, row):
            cell.width = Cm(17 / len(headers))
            paragraph = cell.paragraphs[0]
            paragraph.paragraph_format.space_after = Pt(4)
            paragraph.paragraph_format.space_before = Pt(3)
            paragraph.paragraph_format.line_spacing = 1.05
            add_text(paragraph, str(value))
            for run in paragraph.runs:
                run.font.size = Pt(11)
                if index == 0:
                    run.bold = True
    document.add_paragraph().paragraph_format.space_after = Pt(2)


def render_markdown(document, source):
    lines = source.splitlines()
    index = 0
    while index < len(lines):
        line = lines[index].strip()
        if not line:
            index += 1
            continue
        if line.startswith("```"):
            index += 1
            while index < len(lines) and not lines[index].strip().startswith("```"):
                p = document.add_paragraph(style="Technical Example")
                text_run(p, lines[index])
                index += 1
            index += 1
            continue
        if line.startswith("|") and index + 1 < len(lines) and re.fullmatch(r"[| :\-]+", lines[index + 1].strip()):
            headers = [item.strip() for item in line.strip("|").split("|")]
            index += 2
            rows = []
            while index < len(lines) and lines[index].strip().startswith("|"):
                row = [item.strip() for item in lines[index].strip().strip("|").split("|")]
                if len(row) != len(headers):
                    raise ValueError(f"Invalid table at line {index + 1}")
                rows.append(row)
                index += 1
            add_table(document, headers, rows)
            continue
        heading = re.match(r"^(#{1,3})\s+(.+)$", line)
        if heading:
            p = document.add_paragraph(style=f"Heading {len(heading[1])}")
            add_text(p, heading[2])
            index += 1
            continue
        if re.match(r"^[-*]\s+", line):
            p = document.add_paragraph(style="Manual List")
            add_text(p, "– " + line[2:])
            index += 1
            continue
        parts = [line]
        index += 1
        while index < len(lines) and lines[index].strip() and not re.match(r"^(#{1,3}\s|[-*]\s|\||```)", lines[index].strip()):
            parts.append(lines[index].strip())
            index += 1
        p = document.add_paragraph()
        add_text(p, " ".join(parts))


def configure(document):
    section = document.sections[0]
    section.page_width, section.page_height = Cm(21), Cm(29.7)
    section.left_margin = section.right_margin = Cm(2)
    section.top_margin = Cm(2.1)
    section.bottom_margin = Cm(1.9)
    section.header_distance = section.footer_distance = Cm(0.8)
    for style in document.styles:
        if style.type in (WD_STYLE_TYPE.PARAGRAPH, WD_STYLE_TYPE.CHARACTER, WD_STYLE_TYPE.TABLE):
            font(style.element.get_or_add_rPr())
    normal = document.styles["Normal"]
    normal.font.size = Pt(13)
    normal.paragraph_format.line_spacing = 1.15
    normal.paragraph_format.space_after = Pt(7)
    for name, size in (("Title", 26), ("Subtitle", 16), ("Heading 1", 17), ("Heading 2", 15), ("Heading 3", 13)):
        style = document.styles[name]
        style.font.size = Pt(size)
        style.font.bold = name != "Subtitle"
        style.font.color.rgb = RGBColor(0, 0, 0)
        style.paragraph_format.space_before = Pt(14 if name != "Title" else 0)
        style.paragraph_format.space_after = Pt(8)
        style.paragraph_format.keep_with_next = True
    document.styles["Heading 1"].paragraph_format.page_break_before = True
    for name in ("Hyperlink", "FollowedHyperlink", "TOC 1", "TOC 2", "TOC 3"):
        if name not in document.styles:
            document.styles.add_style(name, WD_STYLE_TYPE.CHARACTER if "Hyperlink" in name else WD_STYLE_TYPE.PARAGRAPH)
        style = document.styles[name]
        font(style.element.get_or_add_rPr(), 12)
        if "Hyperlink" in name:
            style.font.underline = False
    for name, size in (("Technical Example", 11), ("Manual List", 13)):
        style = document.styles.add_style(name, WD_STYLE_TYPE.PARAGRAPH)
        style.base_style = normal
        style.font.size = Pt(size)
        style.paragraph_format.left_indent = Cm(0.3)
        style.paragraph_format.line_spacing = 1.05 if name == "Technical Example" else 1.15
        style.paragraph_format.space_after = Pt(2 if name == "Technical Example" else 5)
    header = section.header.paragraphs[0]
    header.alignment = WD_ALIGN_PARAGRAPH.RIGHT
    text_run(header, "ReconX AI | Tài liệu tổng thể dự án | 0.1.0").font.size = Pt(10)
    footer = section.footer.paragraphs[0]
    footer.alignment = WD_ALIGN_PARAGRAPH.CENTER
    text_run(footer, "Trang ").font.size = Pt(10)
    field = OxmlElement("w:fldSimple")
    field.set(qn("w:instr"), "PAGE")
    run = OxmlElement("w:r")
    props = OxmlElement("w:rPr")
    font(props, 10)
    run.append(props)
    value = OxmlElement("w:t")
    value.text = "1"
    run.append(value)
    field.append(run)
    footer._p.append(field)
    language = OxmlElement("w:lang")
    language.set(qn("w:val"), "vi-VN")
    normal.element.get_or_add_rPr().append(language)
    document.core_properties.title = "ReconX AI — Tài liệu tổng thể dự án"
    document.core_properties.subject = "Kiến trúc, chức năng, hướng dẫn, bảo mật, kiểm thử và trạng thái phát hành"
    document.core_properties.author = "ReconX AI Project"
    document.core_properties.keywords = "ReconX AI, Rust, Tauri, SQLite, tài liệu dự án"
    # Office's inherited theme must not introduce blue headings or another font.
    for part in document.part.package.parts:
        if str(part.partname).startswith("/word/theme/"):
            theme = etree.fromstring(part.blob)
            for element in theme.xpath("//*[local-name()='latin' or local-name()='ea' or local-name()='cs' or local-name()='font']"):
                element.set("typeface", FONT)
            for element in theme.xpath("//*[local-name()='srgbClr']"):
                element.set("val", "000000")
            part._blob = etree.tostring(theme, xml_declaration=True, encoding="UTF-8", standalone=True)


def normalize_fonts(document):
    # stylesWithEffects is a generic OPC part in python-docx, so normalizing
    # only XmlPart.element leaves Office's alternate blue theme styles intact.
    for part in document.part.package.parts:
        if not str(part.partname).startswith("/word/") or not str(part.partname).endswith(".xml"):
            continue
        tree = part.element if hasattr(part, "element") else etree.fromstring(part.blob)
        for run in tree.iter(qn("w:r")):
            props = run.find(qn("w:rPr"))
            if props is None:
                props = OxmlElement("w:rPr")
                run.insert(0, props)
            font(props)
        for props in list(tree.iter(qn("w:rPr"))):
            font(props)
        if not hasattr(part, "element"):
            part._blob = etree.tostring(tree, xml_declaration=True, encoding="UTF-8", standalone=True)


def normalize_saved_document(path):
    """Reapply explicit typography after Word has populated the TOC fields."""
    content = io.BytesIO()
    with zipfile.ZipFile(path) as original, zipfile.ZipFile(content, "w") as output:
        for item in original.infolist():
            data = original.read(item.filename)
            if item.filename.startswith("word/") and item.filename.endswith(".xml"):
                tree = etree.fromstring(data)
                for run in tree.iter(qn("w:r")):
                    props = run.find(qn("w:rPr"))
                    if props is None:
                        props = OxmlElement("w:rPr")
                        run.insert(0, props)
                    font(props)
                for props in list(tree.iter(qn("w:rPr"))):
                    font(props)
                data = etree.tostring(tree, xml_declaration=True, encoding="UTF-8", standalone=True)
            output.writestr(item, data)
    path.write_bytes(content.getvalue())


def appendix(document, binary):
    catalog = json.loads((ROOT / "crates/reconx-ai/src/provider-catalog.json").read_text(encoding="utf-8"))
    document.add_paragraph("Phụ lục A. Danh mục provider từ source", style="Heading 1")
    document.add_paragraph("Bảng được trích từ provider-catalog.json tại thời điểm tạo tài liệu. Đây là preset trong source; không phải xác nhận truy cập tài khoản, tính khả dụng hiện tại hay kết quả test cloud. Trường metadata chưa biết không được tự bổ sung.")
    add_table(document, ["Provider / ID", "Adapter", "Khóa", "Preset model trong source"], [[f'{p["name"]}\n{p["id"]}', p["kind"], "Cần API key" if p["requires_key"] else "Có thể không cần key", "\n".join(m["id"] for m in p["models"]) or "Nhập/discover model local"] for p in catalog])
    document.add_paragraph("Endpoint và nguồn tham khảo theo profile", style="Heading 2")
    for profile in catalog:
        p = document.add_paragraph()
        text_run(p, profile["name"] + ": ", bold=True)
        text_run(p, profile.get("base_url") or "Người dùng cấu hình endpoint")
        document.add_paragraph("Nguồn do registry lưu: " + profile.get("docs_url", "Không có") + "; ngày review trong source: " + profile.get("verified_on", "Không có"), style="Technical Example")
        if profile.get("setup_note"):
            document.add_paragraph(profile["setup_note"])
    document.add_paragraph("Phụ lục B. Migration và các bảng thực tế", style="Heading 1")
    document.add_paragraph("Danh sách tự trích từ SQL. Số 007/008 được giữ lại cho các snapshot phát triển bị gián đoạn; không tái sử dụng hoặc tự xóa bảng lạ trong workspace cũ.")
    for path in sorted((ROOT / "crates/reconx-storage/migrations").glob("*.sql")):
        tables = re.findall(r"CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?([a-zA-Z_][a-zA-Z_0-9]*)", path.read_text(encoding="utf-8"), re.IGNORECASE)
        document.add_paragraph(path.name, style="Heading 2")
        document.add_paragraph("Bảng được tạo: " + (", ".join(tables) or "Không tạo bảng mới"))
    document.add_paragraph("Phụ lục C. Ma trận module từ source", style="Heading 1")
    gaps = json.loads((ROOT / "docs/release-gaps.json").read_text(encoding="utf-8"))
    add_table(document, ["Module", "Trạng thái ghi nhận", "Source chính"], [[m["module"], m["status"], "\n".join(m["source_references"])] for m in gaps["modules"]])
    document.add_paragraph("RUNTIME_VERIFIED trong bảng chỉ áp dụng cho phạm vi hẹp được mô tả trong các chương chức năng và hồ sơ kiểm thử; không đồng nghĩa mọi yêu cầu của module đã hoàn thành.")
    if binary:
        document.add_paragraph("Phụ lục D. CLI reference trích từ executable", style="Heading 1")
        document.add_paragraph("Các đoạn dưới được tạo bằng --help từ executable đã chọn. Không mở database người dùng, không gửi request và không chạy analysis. Ví dụ chi tiết trong chương sử dụng vẫn cần thay ID/path của workspace thực tế.")
        for group in (None, "project", "target", "scope", "reverse", "jobs", "web", "api", "http", "auth", "ai", "tools", "mcp", "workflow", "recon", "evidence", "findings", "report", "settings", "doctor"):
            args = [str(binary), *([group] if group else []), "--help"]
            result = subprocess.run(args, capture_output=True, text=True, encoding="utf-8", timeout=20, check=True)
            document.add_paragraph("reconx " + ((group + " ") if group else "") + "--help", style="Heading 2")
            for line in result.stdout.splitlines():
                text_run(document.add_paragraph(style="Technical Example"), line)


def validate(path):
    """Inspect actual DOCX parts, not just python-docx style objects."""
    issues = []
    text_count = 0
    headings = 0
    with zipfile.ZipFile(path) as package:
        for name in package.namelist():
            if name.endswith(".rels"):
                for rel in etree.fromstring(package.read(name)):
                    if rel.get("TargetMode") == "External":
                        issues.append("External relationship")
            if not name.startswith("word/") or not name.endswith(".xml"):
                continue
            tree = etree.fromstring(package.read(name))
            if "theme" not in name:
                for color in tree.iter(qn("w:color")):
                    if color.get(qn("w:val")) != "000000" or any("theme" in key.lower() for key in color.attrib):
                        issues.append(f"Nonblack color: {name}")
                for fonts in tree.iter(qn("w:rFonts")):
                    if any(value != FONT for key, value in fonts.attrib.items() if key in [qn("w:" + n) for n in ("ascii", "hAnsi", "eastAsia", "cs")]) or any("theme" in key.lower() for key in fonts.attrib):
                        issues.append(f"Wrong font: {name}")
            if name == "word/document.xml":
                text_count = sum(len(node.text or "") for node in tree.iter(qn("w:t")))
                headings = len(tree.xpath("//w:pStyle[starts-with(@w:val,'Heading')]", namespaces={"w": "http://schemas.openxmlformats.org/wordprocessingml/2006/main"}))
                for run in tree.iter(qn("w:r")):
                    if run.find(qn("w:t")) is not None:
                        props = run.find(qn("w:rPr"))
                        if props is None or props.find(qn("w:rFonts")) is None or props.find(qn("w:color")) is None:
                            issues.append("Text without explicit font/color")
    if issues:
        raise ValueError("; ".join(sorted(set(issues))))
    return {"font": FONT, "text_color": "000000", "text_characters": text_count, "heading_count": headings, "xml_validation": "PASS"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=ROOT / "docs/PROJECT_DOCUMENTATION.vi.md")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--binary", type=Path, help="Optional local CLI used only for --help")
    parser.add_argument("--validation-output", type=Path)
    args = parser.parse_args()
    if args.output.exists():
        parser.error("Refusing to overwrite an existing document")
    if args.binary and not args.binary.is_file():
        parser.error("CLI executable does not exist")
    source = args.source.read_text(encoding="utf-8")
    document = Document()
    configure(document)
    cover = document.add_paragraph(style="Title")
    cover.alignment = WD_ALIGN_PARAGRAPH.CENTER
    text_run(cover, "RECONX AI")
    subtitle = document.add_paragraph(style="Subtitle")
    subtitle.alignment = WD_ALIGN_PARAGRAPH.CENTER
    text_run(subtitle, "TÀI LIỆU TỔNG THỂ DỰ ÁN")
    document.add_paragraph("Kiến trúc • Chức năng • Hướng dẫn sử dụng\nBảo mật • Kiểm thử • Vận hành • Phát hành").alignment = WD_ALIGN_PARAGRAPH.CENTER
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    add_table(document, ["Thông tin", "Giá trị"], [["Phiên bản ứng dụng", "0.1.0 — Development build"], ["Mốc tổng hợp", "18/10/2026"], ["Workspace source", r"D:\ReconXAI"], ["Source HEAD lúc biên soạn", revision], ["Trạng thái phát hành", "v1.0 CHƯA HOÀN THÀNH — còn release blockers"], ["Quy cách tài liệu", "DOCX chỉnh sửa được; Times New Roman; toàn bộ chữ màu đen"]])
    document.add_paragraph("Tài liệu mô tả source hiện có và bằng chứng thực thi được lưu cục bộ. Các kết quả test local, CI từ xa, mock-provider, live cloud, Core/CLI và Desktop được phân biệt rõ. Không chứa API key, credential, database người dùng hoặc dữ liệu nghiên cứu riêng tư.")
    document.add_page_break()
    p = document.add_paragraph(style="Heading 1")
    p.paragraph_format.page_break_before = False
    text_run(p, "Mục lục")
    p = document.add_paragraph()
    field = OxmlElement("w:fldSimple")
    field.set(qn("w:instr"), 'TOC \\o "1-2" \\h \\z \\u')
    p._p.append(field)
    render_markdown(document, source)
    appendix(document, args.binary)
    normalize_fonts(document)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("xb") as output:
        document.save(output)
    record = validate(args.output)
    record.update({"output": str(args.output.resolve()), "sha256": hashlib.sha256(args.output.read_bytes()).hexdigest(), "source_revision": revision, "source_sha256": hashlib.sha256(args.source.read_bytes()).hexdigest(), "generated_at": datetime.now(timezone.utc).isoformat(), "application_version": "0.1.0", "v1_release_ready": False})
    if args.validation_output:
        args.validation_output.parent.mkdir(parents=True, exist_ok=True)
        args.validation_output.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    # Windows terminals may still use a legacy code page; the DOCX/JSON files
    # remain Unicode even when a console cannot represent Vietnamese paths.
    print(json.dumps(record, ensure_ascii=True))


if __name__ == "__main__":
    main()
