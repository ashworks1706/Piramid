use serde::Serialize;

#[derive(Serialize)]
pub struct CountResponse {
    pub count: usize,
}
