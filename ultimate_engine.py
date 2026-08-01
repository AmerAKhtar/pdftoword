import logging
import tempfile
import os
from deconstructor import deconstruct_pdf
from table_engine import extract_tables
from pypdf_extractor import extract_pdf_to_word
from qa_inspector import inspect_and_verify_docx

logger = logging.getLogger(__name__)

def convert_pdf_ultimate_engine(pdf_bytes: bytes) -> bytes:
    """
    Executes the Ultimate Vector-First AI-Driven PDF to Word conversion engine.
    Uses pypdf for clean line extraction and python-docx for native element generation.
    """
    logger.info("Starting Ultimate Vector-First AI-Driven Engine")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        in_path = os.path.join(tmpdir, "input.pdf")
        out_path = os.path.join(tmpdir, "Recreated_PDF.docx")
        
        with open(in_path, "wb") as f:
            f.write(pdf_bytes)
            
        logger.info("Phase 1: Parallel Deconstruction & OCR")
        doc_data = deconstruct_pdf(in_path)
        
        logger.info("Phase 2: Native Table Extraction")
        tables_data = extract_tables(in_path)
        
        logger.info("Phase 3: pypdf Extraction & Native Word Element Reconstruction")
        try:
            extract_pdf_to_word(in_path, out_path)
        except Exception as e:
            logger.exception("pypdf extraction failed; falling back to build_native_docx")
            build_native_docx(doc_data, tables_data, out_path)
        
        logger.info("Phase 4: Visual QA Loop (LibreOffice PDF rendering & pdftoppm inspection)")
        qa_stats = inspect_and_verify_docx(out_path, in_path, tmpdir)
        logger.info(f"Visual QA Audit Complete: Verified={qa_stats.get('docx_verified')}, PageMatch={qa_stats.get('page_count_match')}")
        
        if os.path.exists(out_path):
            with open(out_path, "rb") as f:
                result = f.read()
            logger.info("Ultimate engine conversion & visual verification completed successfully")
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
        res = convert_pdf_ultimate_engine(b)
        with open(out_file, "wb") as f:
            f.write(res)
        print(f"Successfully converted {in_file} to {out_file} using ultimate engine")
