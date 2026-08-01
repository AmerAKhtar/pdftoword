import fitz
import pytesseract
from PIL import Image
import io
import concurrent.futures
import logging

logger = logging.getLogger(__name__)

def process_page(pdf_path: str, page_index: int) -> dict:
    """
    Analyzes a single page for layout, text, and runs OCR on images.
    """
    page_data = {
        "page_num": page_index + 1,
        "text_blocks": [],
        "images": [],
        "ocr_text": []
    }
    
    try:
        doc = fitz.open(pdf_path)
        page = doc[page_index]
        page_data["width"] = page.rect.width
        page_data["height"] = page.rect.height
        
        # 1. Native Text Extraction
        blocks = page.get_text("dict").get("blocks", [])
        for block in blocks:
            if block["type"] == 0: # Text
                page_data["text_blocks"].append(block)
            elif block["type"] == 1: # Image
                img_dict = {
                    "bbox": block["bbox"],
                    "image": block.get("image"),
                    "ext": block.get("ext", "png"),
                    "width": block.get("width"),
                    "height": block.get("height")
                }
                page_data["images"].append(img_dict)
                
                # Run OCR on the image
                if img_dict["image"]:
                    try:
                        img = Image.open(io.BytesIO(img_dict["image"]))
                        ocr_result = pytesseract.image_to_data(img, output_type=pytesseract.Output.DICT)
                        # We would parse this deeper in a full implementation to get bounding boxes of OCR text
                        text = pytesseract.image_to_string(img).strip()
                        if text:
                            page_data["ocr_text"].append({
                                "bbox": img_dict["bbox"],
                                "text": text
                            })
                    except Exception as e:
                        logger.warning(f"OCR failed on image on page {page_index + 1}: {e}")
                        
        doc.close()
    except Exception as e:
        logger.error(f"Failed to process page {page_index + 1}: {e}")
        
    return page_data

def deconstruct_pdf(pdf_path: str) -> dict:
    """
    Runs parallel deconstruction of the PDF document.
    """
    logger.info(f"Starting parallel deconstruction of {pdf_path}")
    doc = fitz.open(pdf_path)
    num_pages = len(doc)
    doc.close()
    
    document_data = {"pages": [None] * num_pages}
    
    with concurrent.futures.ProcessPoolExecutor() as executor:
        futures = {executor.submit(process_page, pdf_path, i): i for i in range(num_pages)}
        for future in concurrent.futures.as_completed(futures):
            i = futures[future]
            try:
                document_data["pages"][i] = future.result()
            except Exception as e:
                logger.error(f"Page {i} processing generated an exception: {e}")
                
    return document_data
