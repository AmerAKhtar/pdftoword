import fitz  # PyMuPDF
import logging

logger = logging.getLogger(__name__)

def parse_pdf(pdf_path: str) -> dict:
    """
    Parses a PDF using PyMuPDF to extract text blocks, lines, spans, and images
    with their absolute bounding box coordinates.
    """
    document_data = {
        "pages": []
    }
    
    try:
        doc = fitz.open(pdf_path)
    except Exception as e:
        logger.error(f"Failed to open PDF {pdf_path}: {e}")
        return document_data

    for page_index in range(len(doc)):
        page = doc[page_index]
        page_dict = {
            "page_num": page_index + 1,
            "width": page.rect.width,
            "height": page.rect.height,
            "text_blocks": [],
            "images": []
        }
        
        # Extract text blocks
        blocks = page.get_text("dict").get("blocks", [])
        for block in blocks:
            if block["type"] == 0:  # Text block
                block_dict = {
                    "bbox": block["bbox"],  # (x0, y0, x1, y1)
                    "lines": []
                }
                for line in block.get("lines", []):
                    line_dict = {
                        "bbox": line["bbox"],
                        "dir": line["dir"],
                        "spans": []
                    }
                    for span in line.get("spans", []):
                        span_dict = {
                            "bbox": span["bbox"],
                            "text": span["text"],
                            "font": span["font"],
                            "size": span["size"],
                            "color": span["color"],
                            "flags": span["flags"]
                        }
                        line_dict["spans"].append(span_dict)
                    block_dict["lines"].append(line_dict)
                page_dict["text_blocks"].append(block_dict)
            
            elif block["type"] == 1:  # Image block
                image_dict = {
                    "bbox": block["bbox"],
                    "image": block.get("image"),  # Bytes of the image (PyMuPDF < 1.19 may not have 'image' directly here, let's use page.get_image_info)
                }
                
                # PyMuPDF block type 1 contains 'ext', 'image' bytes etc in some versions.
                # If not, we might need a separate pass for images.
                if "image" in block:
                    image_dict["image"] = block["image"]
                    image_dict["ext"] = block.get("ext", "png")
                    image_dict["width"] = block.get("width")
                    image_dict["height"] = block.get("height")
                    page_dict["images"].append(image_dict)

        document_data["pages"].append(page_dict)
        
    doc.close()
    return document_data
