import pdfplumber
import logging

logger = logging.getLogger(__name__)

def extract_tables(pdf_path: str) -> dict:
    """
    Extracts all native tables from a PDF using pdfplumber's geometric grid detection.
    Returns a dictionary mapping page numbers to a list of tables (2D lists of strings).
    """
    logger.info(f"Extracting native tables from {pdf_path}")
    tables_data = {}
    
    try:
        with pdfplumber.open(pdf_path) as pdf:
            for page_idx, page in enumerate(pdf.pages):
                page_tables = page.extract_tables()
                if page_tables:
                    tables_data[page_idx + 1] = page_tables
    except Exception as e:
        logger.error(f"Failed to extract tables: {e}")
        
    return tables_data
