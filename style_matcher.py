from docx import Document
from docx.shared import Pt, Inches
from docx.enum.style import WD_STYLE_TYPE
import logging

logger = logging.getLogger(__name__)

def generate_styles(doc: Document, text_blocks: list):
    """
    Analyzes all text blocks to cluster fonts and sizes,
    creating native Word styles like Heading 1, Heading 2, Body Text.
    """
    styles = doc.styles
    
    # 1. Analyze fonts and sizes
    size_freq = {}
    for block in text_blocks:
        for line in block.get("lines", []):
            for span in line.get("spans", []):
                size = round(span["size"], 1)
                size_freq[size] = size_freq.get(size, 0) + len(span["text"])
                
    if not size_freq:
        return
        
    # Sort sizes by frequency (most common is likely Body Text)
    sorted_sizes = sorted(size_freq.items(), key=lambda x: x[1], reverse=True)
    body_size = sorted_sizes[0][0]
    
    # Sort all unique sizes ascending to map headings
    unique_sizes = sorted(list(size_freq.keys()))
    
    # 2. Update Body Text
    try:
        body_style = styles['Normal']
        body_style.font.size = Pt(body_size)
    except KeyError:
        pass
        
    # 3. Create Heading styles for anything larger than body_size
    heading_sizes = [s for s in unique_sizes if s > body_size + 1.0]
    heading_sizes.sort(reverse=True) # Largest is Heading 1
    
    for i, size in enumerate(heading_sizes):
        heading_level = i + 1
        if heading_level > 9:
            break
            
        style_name = f'Heading {heading_level}'
        try:
            h_style = styles[style_name]
            h_style.font.size = Pt(size)
        except KeyError:
            # If the template doesn't have it, create it
            # (though python-docx usually has Heading 1-9)
            pass

def match_style(span_size: float, body_size: float) -> str:
    """
    Returns the style name based on the font size relative to body size.
    """
    if span_size > body_size + 10:
        return 'Heading 1'
    elif span_size > body_size + 6:
        return 'Heading 2'
    elif span_size > body_size + 2:
        return 'Heading 3'
    else:
        return 'Normal'
