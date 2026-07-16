use axum::{
    extract::{Path, State},
    body::Body,
    response::{IntoResponse, Response},
    Extension,
};
use chrono::FixedOffset;
use rust_xlsxwriter::*;

use crate::models::*;
use crate::auth::JwtClaims;
use super::{AppError, AppState};

pub async fn export_patient(
    State(state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
) -> Result<ExportResponse, AppError> {
    let patient = state.pool.find_patient_by_id(id).await?
        .ok_or(AppError::NotFound)?;
    let data = state.pool.export_patient_telemetry(id).await?;
    let equivalences = state.pool.load_equivalences().await?;
    let filename = format!("patient_{}_history.xlsx", patient.patient_id_str);
    let bytes = build_excel(&data, &equivalences).map_err(AppError::Export)?;
    Ok(ExportResponse { bytes, filename })
}

pub async fn export_therapy(
    State(state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>,
    Path(id): Path<i64>,
) -> Result<ExportResponse, AppError> {
    let data = state.pool.export_therapy_telemetry(id).await?;
    let equivalences = state.pool.load_equivalences().await?;
    let filename = format!("therapy_{}_data.xlsx", id);
    let bytes = build_excel(&data, &equivalences).map_err(AppError::Export)?;
    Ok(ExportResponse { bytes, filename })
}

fn build_excel(data: &[TelemetryExportRow], equivalences: &[AttributeEquivalence]) -> Result<Vec<u8>, String> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();

    sheet.write_string(0, 0, "ID").map_err(|e| e.to_string())?;
    sheet.write_string(0, 1, "Fecha/Hora").map_err(|e| e.to_string())?;
    sheet.write_string(0, 2, "Señal").map_err(|e| e.to_string())?;
    sheet.write_string(0, 3, "Valor").map_err(|e| e.to_string())?;
    sheet.write_string(0, 4, "Unidad").map_err(|e| e.to_string())?;

    for (i, row) in data.iter().enumerate() {
        let r = (i + 1) as u32;
        sheet.write_number(r, 0, row.id as f64).map_err(|e| e.to_string())?;
        sheet.write_string(
            r,
            1,
            row.timestamp
                .map(|t| t.with_timezone(&FixedOffset::west_opt(5 * 3600).unwrap()).format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default(),
        ).map_err(|e| e.to_string())?;
        if let Some(ref name) = row.signal_name {
            sheet.write_string(r, 2, name).map_err(|e| e.to_string())?;
        } else if let Some(sig) = row.signal_id {
            sheet.write_string(r, 2, sig.to_string()).map_err(|e| e.to_string())?;
        }
        let value_str = row.physical_value.as_deref().unwrap_or("");
        let resolved = row.signal_id
            .and_then(|sid| lookup_equivalence(sid, value_str, equivalences))
            .unwrap_or(value_str);
        if let Ok(n) = resolved.parse::<f64>() {
            sheet.write_number(r, 3, n).map_err(|e| e.to_string())?;
        } else {
            sheet.write_string(r, 3, resolved).map_err(|e| e.to_string())?;
        }
        if let Some(ref unit) = row.unit {
            sheet.write_string(r, 4, unit).map_err(|e| e.to_string())?;
        }
    }

    workbook.save_to_buffer().map_err(|e| e.to_string())
}

pub struct ExportResponse {
    bytes: Vec<u8>,
    filename: String,
}

impl IntoResponse for ExportResponse {
    fn into_response(self) -> Response {
        let safe_filename: String = self.filename.chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '_' || *c == '-')
            .collect();
        let headers = [
            (
                axum::http::header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),
            (
                axum::http::header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", safe_filename),
            ),
        ];
        (
            headers,
            Body::from(self.bytes),
        )
            .into_response()
    }
}
