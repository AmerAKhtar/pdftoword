import cv2
import numpy as np
import io
import logging

logger = logging.getLogger(__name__)

def raster_to_svg(image_bytes: bytes) -> str:
    """
    Converts a raster image to SVG using OpenCV contour detection.
    This works best for logos and solid shapes, but not for complex photographs.
    """
    try:
        # Load image from bytes
        nparr = np.frombuffer(image_bytes, np.uint8)
        img = cv2.imdecode(nparr, cv2.IMREAD_GRAYSCALE)
        
        # Apply threshold to make it binary
        _, thresh = cv2.threshold(img, 127, 255, cv2.THRESH_BINARY_INV)
        
        # Find contours
        contours, hierarchy = cv2.findContours(thresh, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        
        height, width = img.shape
        
        svg_content = f'<svg viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg">\n'
        svg_content += '  <path d="'
        
        for contour in contours:
            if len(contour) < 3:
                continue
                
            # Create SVG path string for each contour
            for i, point in enumerate(contour):
                x, y = point[0]
                if i == 0:
                    svg_content += f"M {x} {y} "
                else:
                    svg_content += f"L {x} {y} "
            svg_content += "Z "
            
        svg_content += '" fill="black" />\n'
        svg_content += '</svg>'
        
        return svg_content
    except Exception as e:
        logger.error(f"Vectorization failed: {e}")
        return ""
