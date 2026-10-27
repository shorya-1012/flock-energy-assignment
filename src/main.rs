mod api;
mod config;
mod portal;
mod state;

use axum::{Json, Router, extract::State, routing::get};
use serde_json::{Value, json};
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

use portal::client::PortalClient;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  dotenvy::dotenv().ok();

  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
    .init();

  let base_url = std::env::var("PORTAL_BASE_URL")?;
  let email = std::env::var("PORTAL_EMAIL")?;
  let password = std::env::var("PORTAL_PASSWORD")?;
  let port: u16 = std::env::var("PORT")
    .unwrap_or_else(|_| "3000".into())
    .parse()?;

  let portal = PortalClient::new(base_url)?;

  tracing::info!("Authenticating with upstream portal");
  portal.login(&email, &password).await?;
  tracing::info!("Portal login succeeded");

  let state = AppState { portal };

  let app = Router::new()
    .route("/health", get(health))
    .route("/meters", get(api::meters::list_meters))
    .route("/meters/{id}/geo", get(api::meters::get_meter_geo))
    .route("/meters/{id}/energy", get(api::meters::get_meter_energy))
    .route("/dts", get(api::dts::list_dts))
    .route("/export", get(api::export::export_meters))
    .with_state(state);

  let address = SocketAddr::from(([127, 0, 0, 1], port));
  let listener = tokio::net::TcpListener::bind(address).await?;

  tracing::info!("API listening on http://{address}");

  axum::serve(listener, app).await?;

  Ok(())
}

async fn health() -> Json<Value> {
  Json(json!({ "status": "ok" }))
}
