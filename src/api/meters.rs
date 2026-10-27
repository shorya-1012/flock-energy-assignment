use crate::state::AppState;
use axum::{
  Json,
  extract::{Path, Query, State},
  http::StatusCode,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::portal::client::PortalClient;

#[derive(Debug, Deserialize)]
pub struct MeterQuery {
  pub q: Option<String>,
  pub page: Option<u32>,
}

pub async fn list_meters(
  State(state): State<AppState>,
  Query(params): Query<MeterQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
  let query = params.q.unwrap_or_default();
  let page = params.page.unwrap_or(1);

  if page == 0 {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(json!({
          "error": "page must be greater than zero"
      })),
    ));
  }

  match state.portal.search_meters(&query, page).await {
    Ok(data) => Ok(Json(json!({
        "data": data
    }))),
    Err(error) => {
      tracing::error!(error = %error, "Meter retrieval failed");

      Err((
        StatusCode::BAD_GATEWAY,
        Json(json!({
            "error": "Unable to retrieve meter data"
        })),
      ))
    }
  }
}

pub async fn get_meter_geo(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
  match state.portal.get_meter_geo(&id).await {
    Ok(data) => Ok(Json(json!({ "data": data }))),
    Err(error) => {
      tracing::error!(error = %error, "Meter geo request failed");

      Err((
        StatusCode::BAD_GATEWAY,
        Json(json!({
            "error": "Unable to retrieve meter coordinates"
        })),
      ))
    }
  }
}

pub async fn get_meter_energy(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
  match state.portal.get_meter_energy(&id).await {
    Ok(data) => Ok(Json(json!({ "data": data }))),
    Err(error) => {
      tracing::error!(error = %error, "Meter energy request failed");

      Err((
        StatusCode::BAD_GATEWAY,
        Json(json!({
            "error": "Unable to retrieve meter energy data"
        })),
      ))
    }
  }
}
