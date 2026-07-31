"""
Hybrid Multi-Engine PDF to Word Synthesizer
Combines LibreOffice headless conversion, Advanced PyMuPDF OCR image extraction,
and PDF layout geometry alignment to generate pixel-matched Word (.docx) documents.
"""

import os
import logging
import tempfile
from docx import Document

from libreoffice_engine import convert_pdf_with_libreoffice
from advanced_ocr_engine import convert_pdf_to_docx_advanced
from fix_alignment import align_docx_with_pdf
from fix_bullets import fix_docx_bullets
from fix_headers_footers import detect_hf_zones, create_redacted_pdf, inject_footer

logger = logging.getLogger(__name__)


def convert_pdf_hybrid_engine(pdf_bytes: bytes, start: int = None, end: int = None) -> bytes:
    """
    Synthesizes LibreOffice, Advanced OCR Image extraction, and PDF geometry alignment
    into a single high-fidelity Word (.docx) document output.
    """
    logger.info("Executing Hybrid Multi-Engine Conversion Pipeline...")

    with tempfile.TemporaryDirectory() as tmp_dir:
        in_pdf = os.path.join(tmp_dir, "input.pdf")
        redacted_pdf = os.path.join(tmp_dir, "input_redacted.pdf")
        lo_docx = os.path.join(tmp_dir, "libreoffice_output.docx")
        adv_docx = os.path.join(tmp_dir, "advanced_ocr_output.docx")
        aligned_docx = os.path.join(tmp_dir, "aligned_output.docx")
        bullets_docx = os.path.join(tmp_dir, "bullets_fixed.docx")
        final_docx = os.path.join(tmp_dir, "final_hybrid_output.docx")

        with open(in_pdf, "wb") as f:
            f.write(pdf_bytes)

        # Step 1: Detect and redact repeating header/footer artifacts
        convert_source = in_pdf
        zones = {"header": None, "footer": None}
        try:
            zones = detect_hf_zones(in_pdf)
            if zones.get("header") or zones.get("footer"):
                create_redacted_pdf(in_pdf, zones, redacted_pdf)
                convert_source = redacted_pdf
        except Exception as hf_err:
            logger.warning(f"Header/footer pre-redaction skipped: {hf_err}")

        with open(convert_source, "rb") as f:
            clean_pdf_bytes = f.read()

        # Step 2: Attempt Engine A (LibreOffice Headless)
        use_lo = False
        try:
            lo_bytes = convert_pdf_with_libreoffice(clean_pdf_bytes)
            with open(lo_docx, "wb") as f:
                f.write(lo_bytes)
            if os.path.exists(lo_docx) and os.path.getsize(lo_docx) > 500:
                use_lo = True
                working_docx = lo_docx
                logger.info("LibreOffice Engine generated baseline DOCX successfully")
        except Exception as lo_err:
            logger.warning(f"LibreOffice engine unavailable/failed ({lo_err}); using Advanced OCR engine")

        # Step 3: Run Engine B (Advanced OCR & High-Res Image Engine)
        if not use_lo:
            try:
                convert_pdf_to_docx_advanced(convert_source, adv_docx)
                working_docx = adv_docx
            except Exception as adv_err:
                logger.exception(f"Advanced OCR engine failed: {adv_err}")
                working_docx = lo_docx if use_lo else adv_docx

        # Step 4: Run Engine C (PDF Geometry Alignment Analyzer & Fixer)
        try:
            align_docx_with_pdf(in_pdf, working_docx, aligned_docx)
            working_docx = aligned_docx
        except Exception as align_err:
            logger.warning(f"Alignment optimization skipped: {align_err}")

        # Step 5: Run Engine D (Bullet Paragraph Normalizer)
        try:
            fix_docx_bullets(working_docx, bullets_docx)
            working_docx = bullets_docx
        except Exception as bullet_err:
            logger.warning(f"Bullet spacing normalization skipped: {bullet_err}")

        # Step 6: Inject clean auto-updating headers/footers if detected
        footer_text = (zones.get("footer") or {}).get("text", "").strip()
        if footer_text:
            try:
                inject_footer(working_docx, footer_text, final_docx)
                working_docx = final_docx
            except Exception as footer_err:
                logger.warning(f"Footer injection skipped: {footer_err}")

        with open(working_docx, "rb") as f:
            output_bytes = f.read()
            logger.info(f"Hybrid Multi-Engine pipeline complete ({len(output_bytes)} bytes)")
            return output_bytes
