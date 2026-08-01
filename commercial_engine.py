import logging
import tempfile
import os
from pdf_parser import parse_pdf
from layout_analyzer import detect_headers_footers
from docx_builder import build_docx_from_data

logger = logging.getLogger(__name__)

def convert_pdf_commercial_engine(pdf_bytes: bytes) -> bytes:
    """
    Executes the Commercial-Grade PDF to Word conversion engine with 100% visual fidelity.
    Uses absolute positioning to replicate the exact PDF layout in Word.
    """
    logger.info("Starting Commercial-Grade Absolute Positioning Engine")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        in_path = os.path.join(tmpdir, "input.pdf")
        out_path = os.path.join(tmpdir, "output.docx")
        
        with open(in_path, "wb") as f:
            f.write(pdf_bytes)
            
        logger.info("Phase 1: Extracting objects with PyMuPDF")
        doc_data = parse_pdf(in_path)
        
        logger.info("Phase 2: Analyzing layout for headers and footers")
        doc_data = detect_headers_footers(doc_data)
        
        logger.info("Phase 3: Building DOCX with OOXML Absolute Positioning")
        build_docx_from_data(doc_data, out_path)
        
        if os.path.exists(out_path):
            with open(out_path, "rb") as f:
                result = f.read()
            logger.info("Commercial engine conversion completed successfully")
            return result
        else:
            raise FileNotFoundError("DOCX generation failed; output file not found.")

if __name__ == "__main__":
    import sys
    if len(sys.argv) > 2:
        in_file = sys.argv[1]
        out_file = sys.argv[2]
        with open(in_file, "rb") as f:
            b = f.read()
        res = convert_pdf_commercial_engine(b)
        with open(out_file, "wb") as f:
            f.write(res)
        print(f"Successfully converted {in_file} to {out_file} using commercial engine")
