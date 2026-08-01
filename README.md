# ConvertFlow — Commercial-Grade Document Understanding Engine (Rust)

ConvertFlow is an enterprise, high-fidelity PDF-to-Word (`.docx`) conversion engine built with a strict separation between low-level PDF parsing, the **Intermediate Document Model (IDM)**, layout understanding, and native Office Open XML (OOXML) rendering.

Unlike traditional direct-mapping converters ($A \rightarrow B$), ConvertFlow treats document conversion as a **semantic reconstruction problem** ($A \rightarrow \text{IDM} \rightarrow B$).

---

## 🏛️ 8-Stage Architecture Breakdown

```
[ PDF Document ]
       │
       ▼
1. PDF Analysis Engine (crates/pdf) ──> Top-Left Coordinate Normalization
       │
       ▼
2. Intermediate Document Model (crates/idm) ──> Abstract AST & Geometry
       │
       ▼
3. Advanced Recovery & OCR Engine (crates/ocr) ──> Scanned Page Recovery
       │
       ▼
4. Layout Understanding Engine (crates/layout) ──> Line Clustering & Heading Classification
       │
       ▼
5. DOCX OOXML Renderer Engine (crates/renderer-docx) ──> WordProcessingML & DrawingML
       │
       ▼
6. Quality Validation Engine (crates/validator) ──> Bounding-Box Drift & Fidelity Scoring
       │
       ▼
7. Concurrency & Pipeline (crates/engine) ──> Rayon Parallel Page Processing
       │
       ▼
8. REST API & Cloud Container (crates/api & Dockerfile) ──> Axum Server (0.0.0.0:8080)
```

---

## 📦 Workspace Package Matrix

| Crate Path | Crate Name | Description | Key Tech / Specs |
| :--- | :--- | :--- | :--- |
| `crates/common` | `common` | Core error enums (`EngineError`) and Result type aliases | `thiserror`, `serde` |
| `crates/geometry` | `geometry` | 2D Point, BoundingBox (with `extend` union), & Transform matrices | 2D Affine Matrix $\mathbf{T} \cdot \mathbf{v}$ |
| `crates/idm` | `idm` | Decoupled Intermediate Document Model AST & Resource Manifest | `serde`, `serde_json` |
| `crates/pdf` | `pdf-parser` | Native PDF stream, glyph, vector shape & image extractor | `pdfium-render`, $\text{Top-Y} = H - Y$ |
| `crates/layout` | `layout-engine` | Line clustering, paragraph aggregation, & semantic classification | $H_1 - H_3$, Lists, Headers/Footers |
| `crates/ocr` | `ocr-engine` | Asynchronous OCR recovery pipeline & deskew manager | `async-trait`, `image` |
| `crates/renderer-docx` | `renderer-docx` | Native Office Open XML (OOXML) package writer | `quick-xml`, `zip` |
| `crates/validator` | `validator` | Bounding-box spatial drift detection & quality fidelity scoring | Quality Score ($0.0 - 1.0$) |
| `crates/engine` | `engine` | End-to-end multi-threaded conversion pipeline orchestrator | `rayon`, `tokio` |
| `crates/api` | `api` | High-throughput Axum REST API web server | `axum`, `tower-http` (CORS) |
| `crates/dependency-manager` | `dependency-manager` | Enterprise SHA-256 dependency verification & binary registry | Crypto integrity verification |

---

## 🚀 Running the Rust Backend API

### Option A: Local Cargo Execution

Make sure you have the Rust toolchain installed (`1.80+`):

```bash
# Build & start the REST API server on http://localhost:8080
cargo run --bin api
```

The service will listen on `http://0.0.0.0:8080` with Permissive CORS enabled.

### Option B: Production Container (Docker / Cloud Run)

Build and launch the containerized release binary:

```bash
# Build multi-stage production image
docker build -t convertflow-api .

# Run container on port 8080
docker run -p 8080:8080 convertflow-api
```

---

## 🌐 Frontend & Live Demo

The static web interface is deployed live on **GitHub Pages**:

🔗 **[https://AmerAKhtar.github.io/pdftoword/](https://AmerAKhtar.github.io/pdftoword/)**

### Features:
- **Interactive Drag & Drop**: Select or drop PDF files.
- **Client-Side PDF Preview**: Real-time rendering via PDF.js.
- **Live DOCX In-Browser Preview**: Render converted `.docx` archives directly in the browser via `docx-preview`.
- **Copyable Error Modal UI**: Selectable and hoverable diagnostic error overlay with 1-click clipboard copy.
- **Configurable API Endpoint**: Easily switch between `http://localhost:8080` or your custom Google Cloud Run endpoint URL.

---

## 🧪 Testing & Verification

Run the comprehensive workspace test suite across all 18 crates:

```bash
cargo test --workspace
```

---

## 📜 API Endpoints

- `GET /health` — Health check endpoint (Returns `OK` 200).
- `POST /v1/convert` — Primary document conversion endpoint (Accepts multipart `file` form-data, returns binary `.docx` archive).
