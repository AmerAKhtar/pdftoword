"""
Fixes a root-cause problem, not a symptom: pdf2docx has no concept of
running headers/footers. It extracts every line of text on a page as body
content, including the same "Name \u2013 Ph: ..." line and page number that
repeat on every page. That's why converted docs show footer text sitting
alone as body paragraphs (sometimes on their own near-blank page) instead
of behaving like a real Word footer that just follows the page.

Approach: don't try to detect and delete footer-shaped body text after
conversion -- redact it out of the PDF *before* pdf2docx ever sees it, then
add it back as a genuine Word footer part. pdf2docx's body ends up clean
because the text is no longer there to extract; the reader still gets a
footer because we build one properly.

Pipeline: detect_hf_zones() -> create_redacted_pdf() -> [pdf2docx runs on
the redacted copy, elsewhere] -> inject_footer() on the resulting docx.
"""

import re
import zipfile
import shutil
from collections import defaultdict
import fitz
from lxml import etree

W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
R = "{http://schemas.openxmlformats.org/officeDocument/2006/relationships}"
REL_NS = "http://schemas.openxmlformats.org/package/2006/relationships"
FOOTER_REL_TYPE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer"


def detect_hf_zones(pdf_path, margin_frac=0.15):
    """Detects a repeating footer (and, symmetrically, header) by finding
    text that appears at the same position across most pages -- the
    defining property of a running header/footer, as opposed to a
    heading or body line that merely happens to sit near a page edge.
    Page numbers are detected separately (pure-digit lines in the footer
    band) since they differ per page and won't match on raw text.
    """
    doc = fitz.open(pdf_path)
    n_pages = len(doc)
    if n_pages < 2:
        doc.close()
        return {"header": None, "footer": None, "page_numbers": {}}

    page_h = doc[0].rect.height
    footer_cut = page_h * (1 - margin_frac)
    header_cut = page_h * margin_frac

    def norm(t):
        return re.sub(r"\d+", "#", t.strip())

    footer_groups, header_groups = defaultdict(list), defaultdict(list)
    page_number_lines = defaultdict(list)

    for page in doc:
        for b in page.get_text("dict")["blocks"]:
            for line in b.get("lines", []):
                y0 = line["bbox"][1]
                text = "".join(s["text"] for s in line["spans"]).strip()
                if not text:
                    continue
                if y0 >= footer_cut:
                    if text.isdigit():
                        page_number_lines[page.number].append(line["bbox"])
                    else:
                        footer_groups[norm(text)].append((page.number, line["bbox"], text))
                elif y0 <= header_cut:
                    header_groups[norm(text)].append((page.number, line["bbox"], text))
    doc.close()

    min_pages = max(2, (n_pages + 1) // 2)

    def pick_repeated(groups):
        best_key, best_count = None, 0
        for key, items in groups.items():
            n = len(set(p for p, _, _ in items))
            if n >= min_pages and n > best_count:
                best_key, best_count = key, n
        return groups[best_key] if best_key else None

    def to_zone(matches):
        if not matches:
            return None
        ys = [b[1] for _, b, _ in matches]
        y1s = [b[3] for _, b, _ in matches]
        return {"text": matches[0][2], "y0": min(ys), "y1": max(y1s), "pages": sorted(set(p for p, _, _ in matches))}

    footer_zone = to_zone(pick_repeated(footer_groups))
    header_zone = to_zone(pick_repeated(header_groups))

    # Fold page-number band into the footer zone if one exists nearby, so
    # redaction covers both in one pass.
    if page_number_lines:
        pn_y0 = min(b[1] for bboxes in page_number_lines.values() for b in bboxes)
        if footer_zone:
            footer_zone["y0"] = min(footer_zone["y0"], pn_y0)
        elif len(page_number_lines) >= min_pages:
            footer_zone = {"text": "", "y0": pn_y0, "y1": page_h, "pages": list(page_number_lines)}

    return {"header": header_zone, "footer": footer_zone, "page_numbers": dict(page_number_lines)}


def create_redacted_pdf(pdf_path, zones, out_path, pad=2):
    """Whites out the detected header/footer bands on every page using
    true redaction (add_redact_annot + apply_redactions), which removes
    the underlying text objects -- not just paints over them -- so
    pdf2docx's text extraction genuinely never sees this content."""
    doc = fitz.open(pdf_path)
    for page in doc:
        if zones.get("footer"):
            y0 = max(0, zones["footer"]["y0"] - pad)
            page.add_redact_annot(fitz.Rect(0, y0, page.rect.width, page.rect.height), fill=(1, 1, 1))
        if zones.get("header"):
            y1 = min(page.rect.height, zones["header"]["y1"] + pad)
            page.add_redact_annot(fitz.Rect(0, 0, page.rect.width, y1), fill=(1, 1, 1))
        page.apply_redactions()
    doc.save(out_path)
    doc.close()


_FOOTER_XML_TEMPLATE = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:p>
    <w:pPr><w:jc w:val="right"/></w:pPr>
    <w:r><w:fldChar w:fldCharType="begin"/></w:r>
    <w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r>
    <w:r><w:fldChar w:fldCharType="end"/></w:r>
  </w:p>
  <w:p>
    <w:r><w:t xml:space="preserve">{text}</w:t></w:r>
  </w:p>
</w:ftr>"""


def inject_footer(docx_path, footer_text, out_path):
    """Adds footer_text (plus a real, auto-updating PAGE field) as a
    genuine Word footer part, and wires it to every section in the
    document so it appears on every page."""
    shutil.copyfile(docx_path, out_path)

    with zipfile.ZipFile(docx_path) as zin:
        names = zin.namelist()
        doc_xml = zin.read("word/document.xml")
        rels_xml = zin.read("word/_rels/document.xml.rels") if "word/_rels/document.xml.rels" in names else None
        ct_xml = zin.read("[Content_Types].xml")

    footer_xml = _FOOTER_XML_TEMPLATE.format(text=footer_text.replace("&", "&amp;").replace("<", "&lt;"))

    # --- relationships: add footer1.xml with a fresh, unused rId ---
    rels_root = etree.fromstring(rels_xml) if rels_xml else etree.fromstring(
        f'<Relationships xmlns="{REL_NS}"/>'.encode()
    )
    existing_ids = {el.get("Id") for el in rels_root}
    n = 1
    while f"rIdFooter{n}" in existing_ids:
        n += 1
    footer_rid = f"rIdFooter{n}"
    rel_el = etree.SubElement(rels_root, f"{{{REL_NS}}}Relationship")
    rel_el.set("Id", footer_rid)
    rel_el.set("Type", FOOTER_REL_TYPE)
    rel_el.set("Target", "footer1.xml")

    # --- content types: declare the new part ---
    ct_root = etree.fromstring(ct_xml)
    CT_NS = "http://schemas.openxmlformats.org/package/2006/content-types"
    override = etree.SubElement(ct_root, f"{{{CT_NS}}}Override")
    override.set("PartName", "/word/footer1.xml")
    override.set(
        "ContentType",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml",
    )

    # --- document.xml: add footerReference to every sectPr ---
    doc_root = etree.fromstring(doc_xml)
    for sectPr in doc_root.iter(f"{W}sectPr"):
        # Remove any prior footer reference of this type before adding ours,
        # so re-running this function is idempotent rather than stacking refs.
        for old in sectPr.findall(f"{W}footerReference"):
            sectPr.remove(old)
        ref = etree.Element(f"{W}footerReference")
        ref.set(f"{W}type", "default")
        ref.set(f"{R}id", footer_rid)
        # footerReference must precede pgSz/pgMar per schema order -- insert first
        sectPr.insert(0, ref)

    new_doc_xml = etree.tostring(doc_root, xml_declaration=True, encoding="UTF-8", standalone=True)
    new_rels_xml = etree.tostring(rels_root, xml_declaration=True, encoding="UTF-8", standalone=True)
    new_ct_xml = etree.tostring(ct_root, xml_declaration=True, encoding="UTF-8", standalone=True)

    with zipfile.ZipFile(docx_path) as zin, zipfile.ZipFile(out_path, "w", zipfile.ZIP_DEFLATED) as zout:
        for item in zin.infolist():
            data = zin.read(item.filename)
            if item.filename == "word/document.xml":
                data = new_doc_xml
            elif item.filename == "word/_rels/document.xml.rels":
                data = new_rels_xml
            elif item.filename == "[Content_Types].xml":
                data = new_ct_xml
            zout.writestr(item, data)
        if "word/_rels/document.xml.rels" not in names:
            zout.writestr("word/_rels/document.xml.rels", new_rels_xml)
        zout.writestr("word/footer1.xml", footer_xml)

    return {"footer_injected": bool(footer_text)}
