use anyhow::{Context, Result, bail};
use hmac::{Hmac, KeyInit, Mac};
use reqwest::{Client, cookie::Jar};
use serde_json::Value;
use sha2::Sha256;
use std::sync::Arc;

fn sign_request(
  method: &str,
  path: &str,
  query: &str,
  secret: &str,
) -> anyhow::Result<(String, String)> {
  let timestamp = chrono::Utc::now().timestamp().to_string();

  let message = format!("{method}\n{path}\n{query}\n{timestamp}");

  let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())?;
  mac.update(message.as_bytes());

  let signature = hex::encode(mac.finalize().into_bytes());

  Ok((timestamp, signature))
}

#[derive(Clone)]
pub struct PortalClient {
  client: Client,
  base_url: String,
}

impl PortalClient {
  pub fn new(base_url: String) -> Result<Self> {
    let jar = Arc::new(Jar::default());

    let client = Client::builder()
      .cookie_provider(jar)
      .build()
      .context("Failed to build HTTP client")?;

    Ok(Self { client, base_url })
  }

  // get session token and storke to cookie jar
  pub async fn login(&self, email: &str, password: &str) -> Result<()> {
    let response = self
      .client
      .post(format!("{}/login", self.base_url))
      .header("Origin", &self.base_url)
      .header("Referer", format!("{}/login", self.base_url))
      .header("x-sveltekit-action", "true")
      .form(&[("email", email), ("password", password)])
      .send()
      .await
      .context("Failed to send portal login request")?;

    let status = response.status();

    let has_session_cookie = response
      .headers()
      .get_all(reqwest::header::SET_COOKIE)
      .iter()
      .any(|cookie| {
        cookie
          .to_str()
          .unwrap_or("")
          .starts_with("__Secure-better-auth.session_token=")
      });

    let body = response
      .text()
      .await
      .context("Failed to read portal login response")?;

    if !status.is_success() || !has_session_cookie {
      bail!(
        "Portal login failed (HTTP {}). \
                 Verify credentials and login response: {}",
        status,
        body.chars().take(300).collect::<String>()
      );
    }

    Ok(())
  }

  // for all meters
  pub async fn search_meters(&self, query: &str, page: u32) -> Result<Value> {
    let response = self
      .client
      .get(format!("{}/portal/meters/search", self.base_url))
      .query(&[("q", query), ("page", &page.to_string())])
      .send()
      .await
      .context("Failed to request meter data")?
      .error_for_status()
      .context("Portal returned an unsuccessful status")?;

    response
      .json::<Value>()
      .await
      .context("Failed to parse meter response as JSON")
  }

  // geo details about specific meter
  pub async fn get_meter_geo(&self, meter_id: &str) -> Result<Value> {
    let response = self
      .client
      .get(format!(
        "{}/portal/meters/{}/geo",
        self.base_url,
        urlencoding::encode(meter_id)
      ))
      .send()
      .await
      .context("Failed to fetch meter coordinates")?
      .error_for_status()
      .context("Portal returned an unsuccessful geo response")?;

    response
      .json::<Value>()
      .await
      .context("Failed to parse meter geo response")
  }

  //details about specific meter
  pub async fn get_meter_energy(&self, meter_id: &str) -> Result<Value> {
    let response = self
      .client
      .get(format!(
        "{}/portal/meters/{}/energy",
        self.base_url,
        urlencoding::encode(meter_id)
      ))
      .send()
      .await
      .context("Failed to fetch meter energy data")?
      .error_for_status()
      .context("Portal returned an unsuccessful energy response")?;

    response
      .json::<Value>()
      .await
      .context("Failed to parse meter energy response")
  }

  // get transformers
  pub async fn get_dts(&self, page: u32) -> Result<Value> {
    let response = self
      .client
      .get(format!("{}/portal/dts", self.base_url))
      .query(&[("page", page)])
      .send()
      .await
      .context("Failed to fetch distribution transformers")?
      .error_for_status()
      .context("Portal returned an unsuccessful DT response")?;

    response
      .json::<Value>()
      .await
      .context("Failed to parse DT response")
  }

  pub async fn export_meters(&self, page: u32) -> anyhow::Result<Value> {
    use anyhow::Context;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct KeysResponse {
      data: KeysData,
    }

    #[derive(Deserialize)]
    struct KeysData {
      #[serde(rename = "signingSecret")]
      signing_secret: String,
    }

    let keys: KeysResponse = self
      .client
      .get(format!("{}/portal/keys", self.base_url))
      .send()
      .await
      .context("Failed to fetch signing key")?
      .error_for_status()
      .context("Portal rejected signing-key request")?
      .json()
      .await
      .context("Failed to parse signing-key response")?;

    let query = format!("page={page}");
    let path = "/portal/export";

    let (timestamp, signature) = sign_request("GET", path, &query, &keys.data.signing_secret)?;

    let response = self
      .client
      .get(format!("{}/portal/export?{query}", self.base_url))
      .header("x-timestamp", timestamp)
      .header("x-signature", signature)
      .send()
      .await
      .context("Failed to send signed export request")?;

    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
      anyhow::bail!("Portal export returned HTTP {status}: {body}");
    }

    Ok(serde_json::from_str(&body)?)
  }
}
