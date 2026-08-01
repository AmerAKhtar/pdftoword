import os
from pypdf import PdfReader
import docx
from docx.shared import Inches, Pt, RGBColor
from docx.enum.text import WD_ALIGN_PARAGRAPH
from docx.oxml import parse_xml
from docx.oxml.ns import nsdecls

def extract_pdf_to_word(pdf_path, output_docx_path):
    """
    Reads a multi-page PDF document and converts its text content into a clean, 
    structured, and fully editable Microsoft Word document (.docx).
    """
    # 1. Read PDF text using pypdf
    reader = PdfReader(pdf_path)
    
    # 2. Initialize Word Document
    doc = docx.Document()

    # Set standard 1-inch margins
    for section in doc.sections:
        section.top_margin = Inches(1.0)
        section.bottom_margin = Inches(1.0)
        section.left_margin = Inches(1.0)
        section.right_margin = Inches(1.0)

    # Helper function for adding custom styled tables
    def add_styled_table(document, data, col_widths):
        table = document.add_table(rows=len(data), cols=len(col_widths))
        table.alignment = WD_ALIGN_PARAGRAPH.CENTER
        for r_idx, row in enumerate(data):
            for c_idx, val in enumerate(row):
                cell = table.cell(r_idx, c_idx)
                cell.text = str(val) if val is not None else ""
                cell.width = col_widths[c_idx]
                p = cell.paragraphs[0]
                p.paragraph_format.space_before = Pt(2)
                p.paragraph_format.space_after = Pt(2)
                if c_idx == 1:
                    p.alignment = WD_ALIGN_PARAGRAPH.RIGHT
                    if p.runs:
                        p.runs[0].font.bold = True
                
                # Add subtle bottom border to table cells using XML manipulation
                tcPr = cell._tc.get_or_add_tcPr()
                tcBorders = parse_xml(r'<w:tcBorders %s><w:bottom w:val="single" w:sz="4" w:space="0" w:color="E0E0E0"/></w:tcBorders>' % nsdecls('w'))
                tcPr.append(tcBorders)
        document.add_paragraph() # Spacing after table

    # 3. Iterate through pages of the PDF and build editable native elements
    for page_idx, page in enumerate(reader.pages):
        text = page.extract_text() or ""
        
        # Add page title / heading based on page content
        p_heading = doc.add_paragraph()
        r_head = p_heading.add_run(f"Page {page_idx + 1} Content Extraction")
        r_head.font.size = Pt(14)
        r_head.font.bold = True
        p_heading.paragraph_format.space_after = Pt(6)

        # Split text lines into paragraphs for native word layout
        lines = text.split('\n')
        for line in lines:
            if line.strip():
                p = doc.add_paragraph()
                p.paragraph_format.line_spacing = 1.15
                p.paragraph_format.space_after = Pt(4)
                r = p.add_run(line.strip())
                r.font.size = Pt(10.5)

        # Add page break between pages (except for the last page)
        if page_idx < len(reader.pages) - 1:
            doc.add_page_break()

    # 4. Save the generated document
    doc.save(output_docx_path)
    print(f"Successfully converted {pdf_path} into {output_docx_path}")

if __name__ == "__main__":
    # Example usage: Change 'input.pdf' to your target file
    input_pdf = "input_document.pdf"
    output_docx = "Recreated_PDF.docx"
    
    if os.path.exists(input_pdf):
        extract_pdf_to_word(input_pdf, output_docx)
    else:
        print(f"File {input_pdf} not found. Please place your PDF in the working directory.")
