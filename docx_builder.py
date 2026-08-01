import io
from docx import Document
from docx.shared import Pt, Inches
from docx.oxml import OxmlElement
from docx.oxml.ns import qn

def pt2emu(pt):
    return int(pt * 12700)

def inject_vml_textbox(paragraph, x_pt, y_pt, width_pt, height_pt, text, font_name, font_size_pt, color):
    """
    Injects a VML Text Box into a python-docx paragraph for absolute positioning.
    VML is older but fully supported and simpler to generate than DrawingML for absolute text boxes.
    """
    # Create the picture wrapper
    pict = OxmlElement('w:pict')
    
    # Create the shape
    shape = OxmlElement('v:shape')
    shape.set('id', f'Text Box')
    shape.set('style', f'position:absolute;mso-position-horizontal-relative:page;mso-position-vertical-relative:page;left:{x_pt}pt;top:{y_pt}pt;width:{width_pt}pt;height:{height_pt}pt;z-index:1;mso-wrap-style:none;v-text-anchor:top')
    shape.set('coordsize', '21600,21600')
    shape.set('o:spt', '202')
    shape.set('path', 'm,l,21600r21600,l21600,xe')
    shape.set('filled', 'f') # No fill
    shape.set('stroked', 'f') # No border
    
    # Create the textbox
    textbox = OxmlElement('v:textbox')
    textbox.set('style', 'mso-fit-shape-to-text:t')
    textbox.set('inset', '0,0,0,0')
    
    # Create the inner content (txbxContent)
    txbxContent = OxmlElement('w:txbxContent')
    
    # Create the paragraph inside the text box
    p = OxmlElement('w:p')
    
    # Add paragraph formatting to remove spacing
    pPr = OxmlElement('w:pPr')
    spacing = OxmlElement('w:spacing')
    spacing.set(qn('w:before'), '0')
    spacing.set(qn('w:after'), '0')
    spacing.set(qn('w:line'), '240')
    spacing.set(qn('w:lineRule'), 'auto')
    pPr.append(spacing)
    p.append(pPr)
    
    # Add the text run
    r = OxmlElement('w:r')
    rPr = OxmlElement('w:rPr')
    
    if font_name:
        rFonts = OxmlElement('w:rFonts')
        rFonts.set(qn('w:ascii'), font_name)
        rFonts.set(qn('w:hAnsi'), font_name)
        rFonts.set(qn('w:cs'), font_name)
        rPr.append(rFonts)
        
    if font_size_pt:
        sz = OxmlElement('w:sz')
        sz.set(qn('w:val'), str(int(font_size_pt * 2)))
        rPr.append(sz)
        
    if color:
        # Convert fitz color tuple (r,g,b) float [0,1] to hex
        if isinstance(color, (list, tuple)) and len(color) == 3:
            hex_color = "{:02X}{:02X}{:02X}".format(int(color[0]*255), int(color[1]*255), int(color[2]*255))
        elif isinstance(color, int):
            hex_color = "{:06X}".format(color)
        else:
            hex_color = "000000"
            
        color_xml = OxmlElement('w:color')
        color_xml.set(qn('w:val'), hex_color)
        rPr.append(color_xml)
    
    t = OxmlElement('w:t')
    t.text = text
    
    r.append(rPr)
    r.append(t)
    p.append(r)
    txbxContent.append(p)
    textbox.append(txbxContent)
    shape.append(textbox)
    pict.append(shape)
    
    paragraph._p.append(pict)

def inject_vml_image(paragraph, doc, x_pt, y_pt, width_pt, height_pt, image_bytes, ext):
    """
    Injects an image absolutely positioned via VML.
    """
    # First, we need to add the image to the document parts
    image_stream = io.BytesIO(image_bytes)
    # Use python-docx internal part to add image
    rel_id, image_part = doc.part.get_or_add_image(image_stream)
    
    pict = OxmlElement('w:pict')
    
    shape = OxmlElement('v:shape')
    shape.set('id', f'Image Box')
    shape.set('style', f'position:absolute;mso-position-horizontal-relative:page;mso-position-vertical-relative:page;left:{x_pt}pt;top:{y_pt}pt;width:{width_pt}pt;height:{height_pt}pt;z-index:-1;mso-wrap-style:none')
    shape.set('coordsize', '21600,21600')
    shape.set('o:spt', '75')
    
    imagedata = OxmlElement('v:imagedata')
    imagedata.set(qn('r:id'), rel_id)
    imagedata.set(qn('o:title'), 'image')
    
    shape.append(imagedata)
    pict.append(shape)
    
    paragraph._p.append(pict)

def build_docx_from_data(document_data: dict, output_path: str):
    """
    Builds a DOCX file from parsed PDF data using absolute positioning.
    """
    doc = Document()
    
    # Basic namespace setup for VML using valid lxml attribute syntax
    doc.element.attrib['{http://www.w3.org/2000/xmlns/}v'] = 'urn:schemas-microsoft-com:vml'
    doc.element.attrib['{http://www.w3.org/2000/xmlns/}o'] = 'urn:schemas-microsoft-com:office:office'
    
    pages = document_data.get("pages", [])
    
    for i, page in enumerate(pages):
        # Set page size
        section = doc.sections[-1]
        section.page_width = Pt(page["width"])
        section.page_height = Pt(page["height"])
        
        # We need zero margins so absolute coordinates match exactly
        section.left_margin = Pt(0)
        section.right_margin = Pt(0)
        section.top_margin = Pt(0)
        section.bottom_margin = Pt(0)
        
        # Create an empty anchor paragraph for the page
        p = doc.add_paragraph()
        pPr = p._p.get_or_add_pPr()
        spacing = OxmlElement('w:spacing')
        spacing.set(qn('w:before'), '0')
        spacing.set(qn('w:after'), '0')
        spacing.set(qn('w:line'), '240')
        spacing.set(qn('w:lineRule'), 'auto')
        pPr.append(spacing)
        
        # Inject Images
        for img in page.get("images", []):
            if "image" in img and img["image"]:
                x0, y0, x1, y1 = img["bbox"]
                w = x1 - x0
                h = y1 - y0
                try:
                    inject_vml_image(p, doc, x0, y0, w, h, img["image"], img.get("ext", "png"))
                except Exception as e:
                    print(f"Failed to inject image: {e}")
            
        # Inject Text
        for block in page.get("text_blocks", []):
            x0, y0, x1, y1 = block["bbox"]
            width = x1 - x0
            height = y1 - y0
            
            # Extract plain text and dominant font for the block
            # For a commercial engine, we'd do this per span, but per block is a good start
            text_content = ""
            font_name = "Arial"
            font_size = 10
            color = 0
            
            for line in block.get("lines", []):
                for span in line.get("spans", []):
                    text_content += span["text"] + " "
                    font_name = span.get("font", font_name)
                    font_size = span.get("size", font_size)
                    color = span.get("color", color)
                text_content = text_content.strip() + "\n"
            text_content = text_content.strip()
            
            if text_content:
                inject_vml_textbox(p, x0, y0, width, height, text_content, font_name, font_size, color)
        
        if i < len(pages) - 1:
            doc.add_page_break()
            
    doc.save(output_path)
