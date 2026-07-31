# DocuShift - PDF to Word Converter

A high-fidelity PDF to Word (`.docx`) converter with a modern glassmorphism web frontend designed for **GitHub Pages** and a Python backend service (`PyMuPDF`, `pdf2docx`, `Flask`).

## 🚀 Features

- **GitHub Pages Ready**: Pure static client (HTML5, Vanilla CSS, Vanilla JS, PDF.js) deployable directly on GitHub Pages.
- **Client-Side PDF Previewer**: Interactive document preview & page navigation powered by PDF.js via CDN.
- **Header & Footer Redaction**: Detects repeating header/footer text across pages and redacts them prior to conversion to eliminate blank page bloat.
- **Bullet & Table Alignment**: Normalizes list spacing and table margins.
- **Custom Page Range**: Convert the entire document or select a custom page range (`start` to `end`).
- **CORS-Enabled API Backend**: Serves local or Cloud Run backends to cross-origin static frontends.

---

## 🌐 Deploying Frontend to GitHub Pages

1. Push this repository to GitHub:
   ```bash
   git add .
   git commit -m "Add GitHub Pages frontend"
   git push origin main
   ```
2. Navigate to your GitHub repository on GitHub.com.
3. Go to **Settings** > **Pages**.
4. Under **Build and deployment**:
   - **Source**: Select `Deploy from a branch`.
   - **Branch**: Select `main` (or `master`) and folder `/ (root)`.
   - Click **Save**.
5. Your frontend site will be live at: `https://<your-username>.github.io/<your-repo-name>/`.

---

## 🐍 Running the Python Backend Service

### Local Setup

1. Install Python dependencies:
   ```bash
   pip install pdf2docx PyMuPDF python-docx fitz Flask
   ```
2. Start the service:
   ```bash
   python pdf_to_docx_service.py
   ```
3. The server will run on `http://localhost:8080`.

### Cloud Run / Server Deployment
You can also deploy the backend using the Dockerfile / Cloud Run specs in `convert_pdf.py` or host it on Render / Railway / AWS.

---

## 🛠️ Usage

1. Open the frontend (either hosted on GitHub Pages or locally opening `index.html`).
2. Ensure the top-right **Backend API URL** field points to your running backend (e.g. `http://localhost:8080` or your Cloud Run URL).
3. Drag and drop your PDF file or click **Select PDF File**.
4. Preview pages and select your desired page range.
5. Click **Convert to Word** and download your converted `.docx` file!
