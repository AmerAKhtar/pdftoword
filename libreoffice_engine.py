"""
LibreOffice Headless Conversion Engine
Converts PDF documents to Word (.docx) format using LibreOffice / soffice CLI.
"""

import os
import shutil
import logging
import subprocess
import tempfile

logger = logging.getLogger(__name__)

# Potential LibreOffice executable locations
SOFFICE_PATHS = [
    "soffice",
    "libreoffice",
    r"C:\Program Files\LibreOffice\program\soffice.exe",
    r"C:\Program Files (x86)\LibreOffice\program\soffice.exe",
    "/usr/bin/soffice",
    "/usr/bin/libreoffice",
    "/Applications/LibreOffice.app/Contents/MacOS/soffice"
]


def find_libreoffice_binary() -> str:
    """Finds the LibreOffice / soffice executable path on the system."""
    for path in SOFFICE_PATHS:
        if shutil.which(path) or (os.path.isabs(path) and os.path.exists(path)):
            return path
    return None


def convert_pdf_with_libreoffice(pdf_bytes: bytes) -> bytes:
    """
    Converts PDF bytes to DOCX bytes using LibreOffice headless command.
    Raises RuntimeError if LibreOffice is not installed or conversion fails.
    """
    binary = find_libreoffice_binary()
    if not binary:
        raise RuntimeError("LibreOffice (soffice) executable not found on host system.")

    logger.info(f"Converting PDF with LibreOffice headless engine using {binary}...")

    with tempfile.TemporaryDirectory() as tmp_dir:
        in_pdf = os.path.join(tmp_dir, "input.pdf")
        with open(in_pdf, "wb") as f:
            f.write(pdf_bytes)

        cmd = [
            binary,
            "--headless",
            "--norestore",
            "--writer",
            "--convert-to", "docx",
            "--outdir", tmp_dir,
            in_pdf
        ]

        try:
            res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=90)
            out_docx = os.path.join(tmp_dir, "input.docx")

            if res.returncode == 0 and os.path.exists(out_docx) and os.path.getsize(out_docx) > 100:
                logger.info(f"LibreOffice conversion successful ({os.path.getsize(out_docx)} bytes)")
                with open(out_docx, "rb") as f:
                    return f.read()
            else:
                stderr_msg = res.stderr.decode("utf-8", errors="ignore")
                raise RuntimeError(f"LibreOffice conversion error (code {res.returncode}): {stderr_msg}")
        except subprocess.TimeoutExpired:
            raise TimeoutError("LibreOffice conversion process timed out after 90 seconds.")
