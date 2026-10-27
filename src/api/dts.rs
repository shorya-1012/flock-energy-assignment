use axum::{
  Json,
  extract::{Query, State},
  http::StatusCode,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct DtQuery {
  pub page: Option<u32>,
}

pub async fn list_dts(
  State(state): State<AppState>,
  Query(params): Query<DtQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
  let page = params.page.unwrap_or(1);

  if page == 0 {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(json!({
          "error": "page must be greater than zero"
      })),
    ));
  }

  match state.portal.get_dts(page).await {
    Ok(data) => Ok(Json(json!({ "data": data }))),
    Err(error) => {
      tracing::error!(error = %error, "DT retrieval failed");

      Err((
        StatusCode::BAD_GATEWAY,
        Json(json!({
            "error": "Unable to retrieve distribution transformers"
        })),
      ))
    }
  }
}
