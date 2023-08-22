use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize)]
pub struct NewPlayerRequest {
    pub(crate) player_link_type: u8,
    pub(crate) player_link_id: Option<String>,
    pub(crate) player_name: String,
}
#[derive(Debug, PartialEq, Serialize)]
pub struct NewPlayerResponse {
    pub(crate) player_id: String,
}

#[derive(Deserialize)]
pub struct AuthRequest {
    pub(crate) player_id: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub(crate) token_id: String,
    pub(crate) auth_response_code: u8,
}
