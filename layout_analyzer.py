import logging
from typing import List, Dict, Set

logger = logging.getLogger(__name__)

def detect_headers_footers(document_data: dict, margin_threshold_pt: float = 72.0) -> dict:
    """
    Analyzes document data to identify repeating elements (headers/footers) across pages.
    margin_threshold_pt defines the top/bottom boundary (e.g. 72pt = 1 inch).
    """
    header_candidates = {}
    footer_candidates = {}
    
    pages = document_data.get("pages", [])
    if len(pages) < 2:
        return document_data  # Not enough pages to detect repeating elements
        
    for page in pages:
        page_num = page["page_num"]
        height = page["height"]
        
        # Analyze Text Blocks
        for block in page.get("text_blocks", []):
            x0, y0, x1, y1 = block["bbox"]
            
            # Extract plain text for hashing/comparison
            text_content = ""
            for line in block.get("lines", []):
                for span in line.get("spans", []):
                    text_content += span["text"]
            
            text_content = text_content.strip()
            if not text_content:
                continue
                
            # Create a signature for this block (text + approx Y position)
            # We use rounded Y to allow for slight sub-pixel variance
            y_sig = round(y0, 1)
            sig = f"{y_sig}_{text_content}"
            
            if y1 < margin_threshold_pt:
                # Potential Header
                if sig not in header_candidates:
                    header_candidates[sig] = {"count": 0, "pages": [], "bbox": block["bbox"], "type": "text", "content": text_content, "block": block}
                header_candidates[sig]["count"] += 1
                header_candidates[sig]["pages"].append(page_num)
                
            elif y0 > (height - margin_threshold_pt):
                # Potential Footer
                if sig not in footer_candidates:
                    footer_candidates[sig] = {"count": 0, "pages": [], "bbox": block["bbox"], "type": "text", "content": text_content, "block": block}
                footer_candidates[sig]["count"] += 1
                footer_candidates[sig]["pages"].append(page_num)

        # Analyze Image Blocks
        for img in page.get("images", []):
            x0, y0, x1, y1 = img["bbox"]
            # Image signature based on dimensions and position
            w = round(img.get("width", 0), 1)
            h = round(img.get("height", 0), 1)
            y_sig = round(y0, 1)
            sig = f"img_{y_sig}_{w}x{h}"
            
            if y1 < margin_threshold_pt:
                if sig not in header_candidates:
                    header_candidates[sig] = {"count": 0, "pages": [], "bbox": img["bbox"], "type": "image", "img": img}
                header_candidates[sig]["count"] += 1
                header_candidates[sig]["pages"].append(page_num)
            elif y0 > (height - margin_threshold_pt):
                if sig not in footer_candidates:
                    footer_candidates[sig] = {"count": 0, "pages": [], "bbox": img["bbox"], "type": "image", "img": img}
                footer_candidates[sig]["count"] += 1
                footer_candidates[sig]["pages"].append(page_num)

    # Filter candidates that appear on more than 1 page
    document_data["headers"] = [v for k, v in header_candidates.items() if v["count"] > 1]
    document_data["footers"] = [v for k, v in footer_candidates.items() if v["count"] > 1]
    
    # Optional: We could remove these blocks from the page_dict to avoid duplication
    
    return document_data
