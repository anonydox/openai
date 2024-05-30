use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum OpenAIError {
    #[error("http error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("{:?}", .0.error)]
    ApiError(ApiErrorResponse),
    #[error("failed to deserialize api response: {0}")]
    JSONDeserialize(serde_json::Error),
    #[error("stream failed: {0}")]
    StreamError(String),
    #[error("invalid args: {0}")]
    InvalidArgument(String),
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorDetail {
    pub message: String,
    pub r#type: String,
    pub param: Option<serde_json::Value>,
    pub code: Option<serde_json::Value>,
}
#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiErrorDetail,
}

pub type OpenAIResponse<T> = Result<T, OpenAIError>;
