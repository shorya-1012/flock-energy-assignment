use crate::portal::client::PortalClient;

#[derive(Clone)]
pub struct AppState {
  pub portal: PortalClient,
}
