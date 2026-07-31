"""
General-purpose PDF -> Word (.docx) conversion for ConvertFlow's Cloud Run
backend, using pdf2docx (PyMuPDF-based extraction + rule-based layout
parsing + python-docx generation).

Bugs fixed here beyond stock pdf2docx:

Bug 1 -- CMYK PNG error ("pixmap must be grayscale or rgb to write as png")
  pdf2docx 0.5.13 correctly converts CMYK *embedded images* via _to_raw_dict,
  but a second code path used for SVG contour detection calls pixmap.tobytes()
  directly, bypassing the colorspace check. Fix: monkeypatch
  clip_page_to_pixmap -- the shared choke-point -- to guarantee RGB output.

Bug 2 -- Excess inter-bullet spacing
  pdf2docx sometimes emits w:before="276"/"326" on bullet paragraphs copied
  from table-based source layouts. fix_bullets.py strips existing w:spacing
  and replaces it with tight values matching the source.

Bug 3 -- Extra blank pages, and footer text leaking into the body
  pdf2docx has no concept of a running header/footer -- it extracts every
  line of text on every page as body content, including the same
  "Name - Ph: ..." line and page number that repeat on every page. That
  shows up as body paragraphs (sometimes alone on a near-blank page) instead
  of a real Word footer. Fix, in order:
    1. detect_hf_zones() finds text that repeats at the same position across
       most pages (the defining trait of a running footer, unlike a heading
       that merely sits near a page edge).
    2. create_redacted_pdf() removes that text from the PDF *before* pdf2docx
       ever runs -- true redaction (add_redact_annot + apply_redactions),
       not a cosmetic white box, so the text genuinely isn't there to extract.
    3. pdf2docx converts the redacted copy, so its body never contains
       footer artifacts in the first place -- no detect-and-delete guessing
       needed downstream.
    4. inject_footer() adds the footer back as a real Word footer part, with
       a genuine auto-updating PAGE field, wired to every section.
  fix_page_bloat() (inside fix_bullets.py) separately cleans up spurious
  w:sectPr elements pdf2docx emits for reconstructed table-column layouts,
  which cause additional blank pages unrelated to the footer issue.
"""

import logging
import os
import tempfile
import fitz
from pdf2docx import Converter
from pdf2docx.image.ImagesExtractor import ImagesExtractor

from fix_bullets import fix_docx_bullets
from fix_headers_footers import detect_hf_zones, create_redacted_pdf, inject_footer
from fix_alignment import align_docx_with_pdf
from advanced_ocr_engine import convert_pdf_to_docx_advanced
from hybrid_engine import convert_pdf_hybrid_engine

logger = logging.getLogger(__name__)


def convert_pdf_to_docx(pdf_bytes: bytes, *, start: int = None, end: int = None) -> bytes:
    """
    Converts PDF bytes to DOCX bytes using Hybrid Multi-Engine pipeline (LibreOffice + Advanced OCR + Layout Alignment).
    """
    try:
        return convert_pdf_hybrid_engine(pdf_bytes, start=start, end=end)
    except Exception as exc:
        logger.exception("Hybrid multi-engine conversion failed; executing emergency fallback")
        with tempfile.TemporaryDirectory() as tmp:
            in_path = os.path.join(tmp, "input.pdf")
            out_path = os.path.join(tmp, "output.docx")
            with open(in_path, "wb") as f:
                f.write(pdf_bytes)
            convert_pdf_to_docx_advanced(in_path, out_path)
            with open(out_path, "rb") as f:
                return f.read()


# ── Flask endpoint ────────────────────────────────────────────────────────────
# `app` must exist at module level -- gunicorn imports this file as a module
# and looks for a top-level `app` attribute. Nesting it inside __main__ means
# gunicorn never sees it.
from flask import Flask, request, send_file, jsonify
import io

app = Flask(__name__, static_folder=".", static_url_path="")


@app.after_request
def add_cors_headers(response):
    response.headers["Access-Control-Allow-Origin"] = "*"
    response.headers["Access-Control-Allow-Headers"] = "Content-Type, Authorization, X-Requested-With"
    response.headers["Access-Control-Allow-Methods"] = "GET, POST, OPTIONS, HEAD"
    return response


@app.route("/", methods=["GET"])
def index():
    return send_file("index.html")


@app.route("/health", methods=["GET"])
def health():
    return jsonify({"status": "ok", "service": "pdf2docx"})


@app.route("/convert/pdf-to-docx", methods=["POST", "OPTIONS"])
def handle_convert():
    if request.method == "OPTIONS":
        return "", 204

    # Extract PDF bytes from multipart form upload or raw body
    pdf_bytes = None
    filename = "converted.docx"
    
    if "file" in request.files:
        file_obj = request.files["file"]
        pdf_bytes = file_obj.read()
        orig_name = file_obj.filename or "document.pdf"
        base_name = os.path.splitext(orig_name)[0]
        filename = f"{base_name}.docx"
    else:
        pdf_bytes = request.get_data()

    if not pdf_bytes:
        return {"error": "No PDF file provided"}, 400

    # Parse optional page range parameters (1-indexed from query/form -> 0-indexed for converter)
    start_arg = request.args.get("start") or request.form.get("start")
    end_arg = request.args.get("end") or request.form.get("end")

    start_idx = None
    end_idx = None
    
    try:
        if start_arg is not None and str(start_arg).strip() != "":
            start_idx = max(0, int(start_arg) - 1)
        if end_arg is not None and str(end_arg).strip() != "":
            end_idx = int(end_arg) # pdf2docx end is 0-indexed non-inclusive or page number
    except ValueError:
        return {"error": "Invalid page range parameters"}, 400

    # Parse optional Adobe API credentials from headers or form inputs
    adobe_client_id = request.headers.get("X-Adobe-Client-Id") or request.form.get("adobe_client_id")
    adobe_client_secret = request.headers.get("X-Adobe-Client-Secret") or request.form.get("adobe_client_secret")

    try:
        docx_bytes = convert_pdf_to_docx(
            pdf_bytes, 
            start=start_idx, 
            end=end_idx,
            client_id=adobe_client_id,
            client_secret=adobe_client_secret
        )
    except Exception as exc:
        logger.exception("pdf2docx conversion failed")
        return {"error": str(exc)}, 422

    return send_file(
        io.BytesIO(docx_bytes),
        mimetype="application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        as_attachment=True,
        download_name=filename,
    )


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=int(os.environ.get("PORT", 8080)))

