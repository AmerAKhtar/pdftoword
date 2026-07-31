"""
Adobe Acrobat Services API Engine
Official Adobe PDF Services REST API integration for converting PDF to Word (.docx).
Uses Adobe Acrobat Cloud Engine with automatic fallback to local high-fidelity engine.
"""

import os
import time
import json
import logging
import urllib.request
import urllib.parse
import urllib.error

logger = logging.getLogger(__name__)

# Adobe PDF Services API Credentials from environment
ADOBE_CLIENT_ID = os.getenv("PDF_SERVICES_CLIENT_ID") or os.getenv("ADOBE_CLIENT_ID")
ADOBE_CLIENT_SECRET = os.getenv("PDF_SERVICES_CLIENT_SECRET") or os.getenv("ADOBE_CLIENT_SECRET")

IMS_TOKEN_URL = "https://pdf-services-ue1.adobe.io/token"
ASSET_URL = "https://pdf-services-ue1.adobe.io/assets"
EXPORT_URL = "https://pdf-services-ue1.adobe.io/operation/exportpdf"


def get_adobe_access_token(client_id: str, client_secret: str) -> str:
    """
    Obtains an OAuth access token from Adobe IMS token endpoint.
    """
    data = urllib.parse.urlencode({
        "client_id": client_id,
        "client_secret": client_secret
    }).encode("utf-8")

    req = urllib.request.Request(
        IMS_TOKEN_URL,
        data=data,
        headers={"Content-Type": "application/x-www-form-urlencoded"}
    )

    with urllib.request.urlopen(req, timeout=15) as resp:
        res_data = json.loads(resp.read().decode("utf-8"))
        return res_data["access_token"]


def convert_pdf_with_adobe_api(pdf_bytes: bytes, client_id: str = None, client_secret: str = None) -> bytes:
    """
    Converts PDF bytes to Word (.docx) bytes using Adobe Acrobat PDF Services Cloud REST API.
    Raises Exception if API credentials are missing or process fails.
    """
    c_id = client_id or ADOBE_CLIENT_ID
    c_secret = client_secret or ADOBE_CLIENT_SECRET

    if not c_id or not c_secret:
        raise ValueError("Adobe Acrobat API Credentials (PDF_SERVICES_CLIENT_ID & PDF_SERVICES_CLIENT_SECRET) not set.")

    logger.info("Connecting to Adobe Acrobat Services API for PDF to Word conversion...")

    # Step 1: Obtain OAuth Access Token
    token = get_adobe_access_token(c_id, c_secret)
    headers = {
        "Authorization": f"Bearer {token}",
        "x-api-key": c_id,
        "Content-Type": "application/json"
    }

    # Step 2: Request Upload Asset URI
    req_body = json.dumps({"mediaType": "application/pdf"}).encode("utf-8")
    req = urllib.request.Request(ASSET_URL, data=req_body, headers=headers, method="POST")
    
    with urllib.request.urlopen(req, timeout=15) as resp:
        asset_res = json.loads(resp.read().decode("utf-8"))
        upload_uri = asset_res["uploadUri"]
        asset_id = asset_res["assetID"]

    # Step 3: Upload PDF File Content to Adobe Asset Storage
    upload_req = urllib.request.Request(
        upload_uri,
        data=pdf_bytes,
        headers={"Content-Type": "application/pdf"},
        method="PUT"
    )
    with urllib.request.urlopen(upload_req, timeout=60) as resp:
        if resp.status not in (200, 201):
            raise RuntimeError(f"Adobe asset upload failed with status {resp.status}")

    # Step 4: Trigger Export PDF to DOCX Operation
    export_body = json.dumps({
        "assetID": asset_id,
        "targetFormat": "docx"
    }).encode("utf-8")

    export_req = urllib.request.Request(EXPORT_URL, data=export_body, headers=headers, method="POST")
    with urllib.request.urlopen(export_req, timeout=15) as resp:
        location_url = resp.headers.get("location")
        if not location_url:
            raise RuntimeError("Adobe export operation did not return job status location header.")

    # Step 5: Poll Job Status until Completion
    poll_headers = {
        "Authorization": f"Bearer {token}",
        "x-api-key": c_id
    }

    download_uri = None
    for _ in range(30):  # Poll for up to 60 seconds
        time.sleep(2)
        poll_req = urllib.request.Request(location_url, headers=poll_headers, method="GET")
        with urllib.request.urlopen(poll_req, timeout=15) as resp:
            status_res = json.loads(resp.read().decode("utf-8"))
            status = status_res.get("status")

            if status == "done":
                download_uri = status_res.get("asset", {}).get("downloadUri") or status_res.get("downloadUri")
                break
            elif status == "failed":
                err_info = status_res.get("error", {})
                raise RuntimeError(f"Adobe Acrobat export failed: {err_info.get('message', 'Unknown error')}")

    if not download_uri:
        raise TimeoutError("Adobe Acrobat API conversion timed out.")

    # Step 6: Download Converted DOCX File Bytes
    dl_req = urllib.request.Request(download_uri, method="GET")
    with urllib.request.urlopen(dl_req, timeout=60) as resp:
        docx_bytes = resp.read()
        logger.info(f"Adobe Acrobat API conversion complete ({len(docx_bytes)} bytes)")
        return docx_bytes
