from docx import Document
from docx.shared import Pt, Inches
from style_matcher import generate_styles, match_style
from vectorizer import raster_to_svg
import logging

logger = logging.getLogger(__name__)

def build_native_docx(document_data: dict, tables_data: dict, output_path: str):
    """
    Reconstructs the document natively using flow layout, native paragraphs, styles, and tables.
    """
    logger.info("Building Native Word Document")
    doc = Document()
    
    # Analyze all text for style generation
    all_text_blocks = []
    for page in document_data.get("pages", []):
        all_text_blocks.extend(page.get("text_blocks", []))
        
    generate_styles(doc, all_text_blocks)
    
    # Calculate baseline body size for style matching
    size_freq = {}
    for block in all_text_blocks:
        for line in block.get("lines", []):
            for span in line.get("spans", []):
                size = round(span["size"], 1)
                size_freq[size] = size_freq.get(size, 0) + len(span["text"])
    
    body_size = 12.0
    if size_freq:
        body_size = sorted(size_freq.items(), key=lambda x: x[1], reverse=True)[0][0]
    
    for i, page in enumerate(document_data.get("pages", [])):
        page_num = i + 1
        
        # Insert Native Tables if any exist for this page
        if page_num in tables_data:
            for table_data in tables_data[page_num]:
                if not table_data: continue
                num_cols = max(len(row) for row in table_data)
                table = doc.add_table(rows=len(table_data), cols=num_cols)
                table.style = 'Table Grid'
                for r_idx, row in enumerate(table_data):
                    for c_idx, cell_text in enumerate(row):
                        if cell_text:
                            table.cell(r_idx, c_idx).text = str(cell_text)
        
        # Flow Text Blocks
        for block in page.get("text_blocks", []):
            x0 = block["bbox"][0]
            
            # Combine text to evaluate style
            text_content = ""
            max_size = 0
            for line in block.get("lines", []):
                for span in line.get("spans", []):
                    text_content += span["text"] + " "
                    max_size = max(max_size, span["size"])
            
            text_content = text_content.strip()
            if not text_content: continue
            
            style_name = match_style(max_size, body_size)
            p = doc.add_paragraph(text_content, style=style_name)
            
            # Replicate indentation programmatically
            # Assuming 72pt = 1 inch
            indent_inches = x0 / 72.0
            if indent_inches > 0.5: # Only apply significant indents to avoid chaos
                p.paragraph_format.left_indent = Inches(indent_inches - 0.5)
        
        # OCR Text (Flowing as normal text since it shouldn't be rasterized)
        for ocr in page.get("ocr_text", []):
            doc.add_paragraph(ocr["text"], style='Normal')
            
        # Vectorize graphics (SVGs can't natively be embedded easily with pure python-docx without OOXML injection)
        # For this engine, we will log the vectorization success.
        for img in page.get("images", []):
            if img.get("image"):
                svg = raster_to_svg(img["image"])
                if svg:
                    logger.info("Successfully vectorized image into SVG")
                    # Real implementation would write SVG out and link it via lxml.
                    
        if i < len(document_data.get("pages", [])) - 1:
            doc.add_page_break()
            
    doc.save(output_path)
