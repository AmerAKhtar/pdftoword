use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use engine::ConversionPipeline;
use ocr_engine::tesseract_provider::TesseractOcrProvider;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tower_http::cors::{Any, CorsLayer};

struct AppState {
    pipeline: ConversionPipeline,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let ocr = Box::new(TesseractOcrProvider::new());
    let pipeline = ConversionPipeline::new(ocr).expect("Failed to initialize engine");

    let state = Arc::new(AppState { pipeline });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route(
            "/",
            axum::routing::get(|| async {
                Json(json!({
                    "service": "ConvertFlow PDF-to-Word Conversion Engine",
                    "status": "online",
                    "version": "1.0.0",
                    "endpoints": {
                        "health": "GET /health",
                        "convert": "POST /v1/convert",
                        "legacy_convert": "POST /convert/pdf-to-docx"
                    }
                }))
            }),
        )
        .route("/v1/convert", post(convert_document_handler))
        .route("/convert/pdf-to-docx", post(convert_document_handler))
        .route("/health", axum::routing::get(|| async { "OK" }))
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:8080";
    tracing::info!("ConvertFlow Engine listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn convert_document_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let temp_pdf = NamedTempFile::new().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let output_docx = PathBuf::from(format!("{}.docx", temp_pdf.path().display()));

    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            std::fs::write(temp_pdf.path(), data).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
    }

    let report = state
        .pipeline
        .convert_pdf_to_docx(temp_pdf.path(), &output_docx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = std::fs::remove_file(&output_docx);

    Ok(Json(json!({
        "status": "success",
        "quality_score": report.overall_quality_score,
        "text_fidelity": report.text_fidelity_score,
        "deviations_count": report.deviations.len()
    })))
}
