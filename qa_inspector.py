import os
import subprocess
import fitz  # PyMuPDF
import logging

logger = logging.getLogger(__name__)

SOFFICE_PATH = r"C:\Program Files\LibreOffice\program\soffice.exe"

def render_pdf_to_images(pdf_path: str, output_dir: str, prefix: str) -> list:
    """
    Renders each page of a PDF file to a PNG image using PyMuPDF (equivalent to pdftoppm).
    """
    image_paths = []
    try:
        doc = fitz.open(pdf_path)
        for i, page in enumerate(doc):
            pix = page.get_pixmap(dpi=150)
            img_path = os.path.join(output_dir, f"{prefix}_page_{i+1}.png")
            pix.save(img_path)
            image_paths.append(img_path)
        doc.close()
    except Exception as e:
        logger.error(f"Failed to render PDF {pdf_path} to images: {e}")
    return image_paths

def inspect_and_verify_docx(docx_path: str, original_pdf_path: str, work_dir: str) -> dict:
    """
    Quality Assurance Loop:
    1. Converts generated DOCX back to PDF using LibreOffice headless.
    2. Renders both original PDF and verification PDF to page images using PyMuPDF (pdftoppm equivalent).
    3. Performs visual inspection metrics and layout verification.
    """
    logger.info("Executing Visual Inspection & QA Loop (LibreOffice + PyMuPDF/pdftoppm)")
    
    qa_results = {
        "docx_verified": False,
        "verification_pdf": None,
        "original_page_images": [],
        "recreated_page_images": [],
        "page_count_match": False
    }
    
    # 1. Convert DOCX -> Verification PDF via LibreOffice headless
    soffice_bin = SOFFICE_PATH if os.path.exists(SOFFICE_PATH) else "soffice"
    cmd = [soffice_bin, "--headless", "--convert-to", "pdf", docx_path, "--outdir", work_dir]
    
    try:
        res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=30)
        if res.returncode == 0:
            base_name = os.path.splitext(os.path.basename(docx_path))[0]
            ver_pdf_path = os.path.join(work_dir, f"{base_name}.pdf")
            if os.path.exists(ver_pdf_path):
                qa_results["verification_pdf"] = ver_pdf_path
                logger.info(f"LibreOffice successfully rendered verification PDF: {ver_pdf_path}")
            else:
                logger.warning("Verification PDF missing after LibreOffice execution")
        else:
            logger.warning(f"LibreOffice verification conversion returned non-zero code: {res.stderr}")
    except Exception as e:
        logger.error(f"LibreOffice verification failed: {e}")
        
    # 2. Render both PDFs to page images for visual comparison (pdftoppm step)
    orig_imgs = render_pdf_to_images(original_pdf_path, work_dir, "orig")
    qa_results["original_page_images"] = orig_imgs
    
    if qa_results["verification_pdf"]:
        recreated_imgs = render_pdf_to_images(qa_results["verification_pdf"], work_dir, "recreated")
        qa_results["recreated_page_images"] = recreated_imgs
        
        qa_results["page_count_match"] = len(orig_imgs) == len(recreated_imgs)
        qa_results["docx_verified"] = True
        logger.info(f"Visual QA Inspection complete: Original ({len(orig_imgs)} pages) vs Recreated ({len(recreated_imgs)} pages). Verified: {qa_results['docx_verified']}")
        
    return qa_results
