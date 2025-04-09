use serde::Deserialize;

#[derive(Deserialize)]
pub struct WsParams {
    pub filename: String,
}
