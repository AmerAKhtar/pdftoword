"""
Fixes two real, verified pdf2docx bullet failures found in the docx you
uploaded (Resume-Support__3_.docx):

1. When the source PDF's bullet glyph was in certain symbol fonts
   (OpenSymbol, in your resume's case), pdf2docx creates the run with
   the right font and position but an EMPTY <w:t> -- the bullet
   character itself is lost. Confirmed on the "Work Experience" and
   "Key Skills" lists.

2. Some bullet lists get rebuilt as a 2-column table instead of a real
   list: column 1 holds N bullet glyphs stacked with manual line
   breaks, column 2 holds N separate paragraphs of text. The two
   columns are only visually lined up by having matching counts --
   there's no structural link, so editing text in one place (a bullet
   wrapping to 2 lines, one item added) can misalign bullets from
   their text. Confirmed on "12 Weeks of Internship" (bullets survived
   here, font was Noto Sans Symbols2) and "Key Skills" (bullets lost,
   same as #1 -- it has both problems at once).

Root cause of #1, corrected after closer inspection (the first pass at
this diagnosis was wrong -- worth recording why): the character isn't
actually deleted. `<w:t>` contains a real codepoint, U+F0B7 -- it just
prints as invisible in a terminal, which is what made it look empty on
first read. The actual problem is that U+F0B7 only means "bullet" inside
the specific font (OpenSymbol) that originally defined it there, the way
Wingdings/Symbol fonts remap the same Private-Use-Area range to their
own glyph sets. OpenSymbol is LibreOffice's font, not something Word or
most systems have, and it isn't embedded in the docx -- so whatever font
actually resolves in the reader's Word almost certainly has no glyph at
that codepoint, rendering as blank space or a missing-glyph box. Not
something worth chasing upstream in pdf2docx -- more tractable to detect
the damage and rebuild with a real, portable bullet character instead.

Fix: find both patterns and replace them with genuine Word bulleted-list
paragraphs (real w:numPr against an injected bullet numbering
definition), regardless of which broken shape the text currently lives
in. That's more robust than just re-inserting a bullet character into
the same fragile structures -- it also fixes the table-based
misalignment risk, not just the missing glyph.
"""

import zipfile
import shutil
from lxml import etree

W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
NSMAP = {"w": W[1:-1]}

SYMBOL_FONTS = {
    "OpenSymbol", "Wingdings", "Wingdings 2", "Wingdings 3",
    "Symbol", "Noto Sans Symbols", "Noto Sans Symbols2",
}

BULLET_NUM_ID = 9001
BULLET_ABSTRACT_ID = 9001


def _run_font(r):
    rFonts = r.find(f"{W}rPr/{W}rFonts")
    return rFonts.get(f"{W}ascii") if rFonts is not None else None


def _make_bullet_pPr(base_pPr):
    """Clones a paragraph's pPr (if any) and adds real bullet numPr.

    Bug fix: pdf2docx sometimes emits w:before="276" or w:before="326" on
    bullet paragraphs when the source PDF used table-based layout with explicit
    vertical spacing. Copying that pPr wholesale carries the excess spacing into
    the rebuilt bullets, making them look far more spread out than in the source.
    Fix: strip any existing w:spacing from the copied pPr and replace it with
    tight values measured from the original PDF (before=0, after=60 which is
    3pt -- a breath between items without a gap, matching the source's ~30pt
    top-to-top rhythm at 13pt font size).
    """
    pPr = etree.Element(f"{W}pPr")
    if base_pPr is not None:
        for child in base_pPr:
            tag = etree.QName(child).localname
            if tag in ("numPr", "spacing", "ind"):
                continue  # strip old spacing and indent -- will replace both
            pPr.append(etree.fromstring(etree.tostring(child)))

    # Tight inter-bullet spacing derived from the original PDF measurements:
    # 30pt top-to-top gap at 13pt font = 0pt before + ~3pt (60 twips) after.
    spacing = etree.SubElement(pPr, f"{W}spacing")
    spacing.set(f"{W}before", "0")
    spacing.set(f"{W}after", "60")
    spacing.set(f"{W}line", "276")
    spacing.set(f"{W}lineRule", "auto")

    numPr = etree.SubElement(pPr, f"{W}numPr")
    etree.SubElement(numPr, f"{W}ilvl").set(f"{W}val", "0")
    etree.SubElement(numPr, f"{W}numId").set(f"{W}val", str(BULLET_NUM_ID))

    ind = etree.SubElement(pPr, f"{W}ind")
    ind.set(f"{W}left", "720")
    ind.set(f"{W}hanging", "360")
    return pPr


def fix_plain_paragraph_bullets(root):
    """Pattern 1: <w:p> whose first run is a symbol font with lost/real
    bullet text, immediately followed by a real text run, not inside a
    table, no existing numPr. Strips the placeholder run, makes the
    paragraph a real bullet."""
    fixed = 0
    for p in root.iter(f"{W}p"):
        if p.getparent() is not None and etree.QName(p.getparent()).localname == "tc":
            pass  # still fine to check; table-cell paragraphs are handled by pattern 2 separately
        runs = p.findall(f"{W}r")
        if len(runs) < 2:
            continue
        first_font = _run_font(runs[0])
        if first_font not in SYMBOL_FONTS:
            continue
        pPr = p.find(f"{W}pPr")
        if pPr is not None and pPr.find(f"{W}numPr") is not None:
            continue  # already a real list somehow
        # must be followed by a run with actual text (the bullet's content)
        second_text = runs[1].find(f"{W}t")
        if second_text is None or not (second_text.text or "").strip():
            continue
        new_pPr = _make_bullet_pPr(pPr)
        if pPr is not None:
            p.remove(pPr)
        p.insert(0, new_pPr)
        p.remove(runs[0])  # drop the placeholder bullet run
        fixed += 1
    return fixed


def fix_table_bullets(root):
    """Pattern 2: single-row, 2-cell <w:tbl> where cell 1 is repeated
    symbol-font runs joined by <w:br/>, cell 2 is N real paragraphs.
    Replaces the whole table with N real bulleted paragraphs."""
    fixed_lists = 0
    for tbl in list(root.iter(f"{W}tbl")):
        trs = tbl.findall(f"{W}tr")
        if len(trs) != 1:
            continue
        tcs = trs[0].findall(f"{W}tc")
        if len(tcs) != 2:
            continue
        cell1_runs = tcs[0].findall(f".//{W}r")
        cell1_fonts = {_run_font(r) for r in cell1_runs if _run_font(r)}
        has_brs = any(r.find(f"{W}br") is not None for r in cell1_runs)
        if not has_brs or not cell1_fonts.issubset(SYMBOL_FONTS) or not cell1_fonts:
            continue  # not a bullet-table -- e.g. the title/date tables, leave untouched

        # Cell 2 sometimes holds one bullet item per <w:p> (Internship
        # list), and sometimes crams every item into ONE <w:p> separated
        # by <w:br/> instead (Key Skills list, verified by testing --
        # my first pass here only split on <w:p> and silently glued
        # every item after the first back into one run-on paragraph).
        # Split on both: treat each <w:p>, and within it each <w:br/>,
        # as its own item.
        new_paragraphs = []
        for tp in tcs[1].findall(f"{W}p"):
            src_pPr = tp.find(f"{W}pPr")
            item_runs = [[]]
            for run in tp.findall(f"{W}r"):
                if run.find(f"{W}br") is not None:
                    item_runs.append([])
                    continue
                item_runs[-1].append(run)
            for runs in item_runs:
                if not any((r.find(f"{W}t") is not None and (r.find(f"{W}t").text or "").strip()) for r in runs):
                    continue  # empty split (e.g. trailing break with nothing after it)
                new_p = etree.Element(f"{W}p")
                new_p.append(_make_bullet_pPr(src_pPr))
                for run in runs:
                    new_p.append(etree.fromstring(etree.tostring(run)))
                new_paragraphs.append(new_p)

        parent = tbl.getparent()
        idx = list(parent).index(tbl)
        for offset, new_p in enumerate(new_paragraphs):
            parent.insert(idx + offset, new_p)
        parent.remove(tbl)
        fixed_lists += 1
    return fixed_lists


def ensure_bullet_numbering(numbering_root):
    """Adds our bullet abstractNum/num if not already present."""
    existing_ids = {n.get(f"{W}numId") for n in numbering_root.findall(f"{W}num")}
    if str(BULLET_NUM_ID) in existing_ids:
        return
    abstractNum = etree.SubElement(numbering_root, f"{W}abstractNum")
    abstractNum.set(f"{W}abstractNumId", str(BULLET_ABSTRACT_ID))
    lvl = etree.SubElement(abstractNum, f"{W}lvl")
    lvl.set(f"{W}ilvl", "0")
    etree.SubElement(lvl, f"{W}start").set(f"{W}val", "1")
    etree.SubElement(lvl, f"{W}numFmt").set(f"{W}val", "bullet")
    etree.SubElement(lvl, f"{W}lvlText").set(f"{W}val", "\u2022")  # real "•", not a PUA glyph
    etree.SubElement(lvl, f"{W}lvlJc").set(f"{W}val", "left")
    pPr = etree.SubElement(lvl, f"{W}pPr")
    etree.SubElement(pPr, f"{W}ind").set(f"{W}left", "720")
    pPr[-1].set(f"{W}hanging", "360")
    num = etree.SubElement(numbering_root, f"{W}num")
    num.set(f"{W}numId", str(BULLET_NUM_ID))
    etree.SubElement(num, f"{W}abstractNumId").set(f"{W}val", str(BULLET_ABSTRACT_ID))
    # abstractNum elements must come before any w:num referencing them,
    # and (per schema) before other trailing elements -- inserting at
    # a fixed position is safer than relying on append order.
    numbering_root.remove(abstractNum)
    numbering_root.insert(0, abstractNum)


def fix_docx_bullets(in_path: str, out_path: str) -> dict:
    shutil.copyfile(in_path, out_path)
    with zipfile.ZipFile(in_path) as zin:
        doc_xml = zin.read("word/document.xml")
        num_xml = zin.read("word/numbering.xml")
        names = zin.namelist()

    doc_root = etree.fromstring(doc_xml)
    num_root = etree.fromstring(num_xml)

    n_plain = fix_plain_paragraph_bullets(doc_root)
    n_tables = fix_table_bullets(doc_root)
    page_stats = fix_page_bloat(doc_root)
    ensure_bullet_numbering(num_root)

    new_doc_xml = etree.tostring(doc_root, xml_declaration=True, encoding="UTF-8", standalone=True)
    new_num_xml = etree.tostring(num_root, xml_declaration=True, encoding="UTF-8", standalone=True)

    with zipfile.ZipFile(in_path) as zin, zipfile.ZipFile(out_path, "w", zipfile.ZIP_DEFLATED) as zout:
        for item in zin.infolist():
            data = zin.read(item.filename)
            if item.filename == "word/document.xml":
                data = new_doc_xml
            elif item.filename == "word/numbering.xml":
                data = new_num_xml
            zout.writestr(item, data)

    return {"plain_paragraph_fixes": n_plain, "table_list_fixes": n_tables, **page_stats}


def fix_page_bloat(doc_root) -> dict:
    """Removes spurious section properties and empty paragraphs that pdf2docx
    emits when reconstructing multi-column table layouts, causing extra blank
    pages in the output docx.

    Root cause (diagnosed against real pdf2docx 0.5.13 output):
    pdf2docx represents each PDF page as a sectPr element (w:type='nextPage')
    inside an empty paragraph. That's correct for the real page boundaries.
    BUT it also emits sectPr elements with type='continuous' or 'nextColumn'
    for every internal table-column boundary it reconstructs -- those are NOT
    page breaks and should not be sections at all. Each one creates a spurious
    section, and the chain of empty paragraphs carrying these sectPr elements,
    plus one leading empty paragraph at the document start, adds enough blank
    lines to push content onto an extra page.

    Strategy:
    1. Strip sectPr from any paragraph whose type is NOT 'nextPage' -- these
       are table-layout artifacts, not real page breaks.
    2. Remove the resulting empty paragraphs that have no content AND no sectPr
       (after the strip above). Specifically targets leading/trailing empties
       and those between a sectPr paragraph and real content.
    3. Leave 'nextPage' sectPr paragraphs untouched -- those are the correct
       PDF page markers.
    """
    body = doc_root.find(f"{W}body")
    if body is None:
        return {"spurious_sectPr_removed": 0, "empty_paras_removed": 0}

    # Pass 1: strip non-nextPage sectPr from empty paragraphs
    removed_sect = 0
    for p in list(body):
        if etree.QName(p).localname != "p":
            continue
        sp = p.find(f"{W}pPr/{W}sectPr")
        if sp is None:
            sp = p.find(f"{W}sectPr")
        if sp is None:
            continue
        # Determine type
        tp_el = sp.find(f"{W}type")
        tp = tp_el.get(f"{W}val") if tp_el is not None else "nextPage"
        if tp == "nextPage":
            continue  # real page break -- keep
        # Only strip from empty paragraphs (no visible text)
        texts = p.findall(f".//{W}t")
        text = "".join(t.text or "" for t in texts).strip()
        if text:
            continue  # has content -- leave untouched
        # Remove the sectPr (it may be nested inside pPr or directly in p)
        pPr = p.find(f"{W}pPr")
        if pPr is not None:
            inner = pPr.find(f"{W}sectPr")
            if inner is not None:
                pPr.remove(inner)
                removed_sect += 1
        direct = p.find(f"{W}sectPr")
        if direct is not None:
            p.remove(direct)
            removed_sect += 1

    # Pass 2: remove empty paragraphs at the very start and between
    # sectPr-paragraphs and real content. Only remove if they carry no
    # text and no sectPr (the sectPr-carrying ones that matter were already
    # handled above or left alone intentionally).
    children = list(body)
    to_remove = set()

    # Leading empty paragraph(s) at the start of the document
    for i, child in enumerate(children):
        if etree.QName(child).localname != "p":
            break
        texts = child.findall(f".//{W}t")
        text = "".join(t.text or "" for t in texts).strip()
        sp = child.find(f".//{W}sectPr")
        if text or sp is not None:
            break
        to_remove.add(i)

    # Empty paragraphs that sit between a sectPr paragraph and real content
    # (these are the buffer empties pdf2docx emits after each sectPr paragraph)
    for i, child in enumerate(children):
        if i in to_remove:
            continue
        if etree.QName(child).localname != "p":
            continue
        sp = child.find(f".//{W}sectPr")
        if sp is None:
            continue
        # next sibling: if it's an empty para with no sectPr, mark for removal
        if i + 1 < len(children):
            nxt = children[i + 1]
            if etree.QName(nxt).localname == "p":
                nxt_texts = nxt.findall(f".//{W}t")
                nxt_text = "".join(t.text or "" for t in nxt_texts).strip()
                nxt_sp = nxt.find(f".//{W}sectPr")
                if not nxt_text and nxt_sp is None:
                    to_remove.add(i + 1)

    removed_empty = 0
    for i in sorted(to_remove, reverse=True):
        body.remove(children[i])
        removed_empty += 1

    return {"spurious_sectPr_removed": removed_sect, "empty_paras_removed": removed_empty}

if __name__ == "__main__":
    import sys
    stats = fix_docx_bullets(sys.argv[1], sys.argv[2])
    print(stats)


