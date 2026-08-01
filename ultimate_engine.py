import logging
import tempfile
import os
from deconstructor import deconstruct_pdf
from table_engine import extract_tables
from native_builder import build_native_docx

logger = logging.getLogger(__name__)

def convert_pdf_ultimate_engine(pdf_bytes: bytes) -> bytes:
    """
    Executes the Ultimate Vector-First AI-Driven PDF to Word conversion engine.
    Prioritizes native flow-layout, AI style matching, and vector graphics.
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
        
        logger.info("Phase 3: Vector-First Formatting & Native Reconstruction")
        build_native_docx(doc_data, tables_data, out_path)
        
        if os.path.exists(out_path):
            with open(out_path, "rb") as f:
                result = f.read()
            logger.info("Ultimate engine conversion completed successfully")
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
