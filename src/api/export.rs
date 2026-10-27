use axum::{
  Json,
  extract::{Query, State},
  http::StatusCode,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct ExportQuery {
  pub page: Option<u32>,
}

pub async fn export_meters(
  State(state): State<AppState>,
  Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
  let page = params.page.unwrap_or(1);

  if page == 0 {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(json!({"error": "page must be greater than zero"})),
    ));
  }

  match state.portal.export_meters(page).await {
    Ok(data) => Ok(Json(data)),
    Err(error) => {
      tracing::error!(
          error = ?error,
          "Meter export failed"
      );

      Err((
        StatusCode::BAD_GATEWAY,
        Json(json!({
            "error": "Unable to export meter data",
        })),
      ))
    }
  }
}
