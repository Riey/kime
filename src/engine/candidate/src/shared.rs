use serde::{Deserialize, Serialize};

pub const CANDIDATE_PROCESS_NAME: &str = "kime-candidate-window";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitRequest {
    pub candidate_list: Vec<(String, String)>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Request {
    Close,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
    Selected(String),
    Quit,
}
