use crate::api::extraction_prompt::PromptTemplate;
use derive_builder::Builder;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::client::{ClientEnum, ReqClient};
use crate::core::response_wrapper::{OpenAIError, OpenAIResponse};
use crate::core::types::Stop;
#[derive(Debug, Serialize, Deserialize, Clone, Default, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    #[strum(serialize = "system")]
    System,
    #[default]
    #[strum(serialize = "user")]
    User,
    #[strum(serialize = "assistant")]
    Assistant,
}

#[derive(Builder, Default, Debug, Clone, Deserialize, Serialize)]
#[builder(name = "ChatCompletionMessageRequestBuilder")]
#[builder(pattern = "mutable")]
#[builder(setter(into, strip_option), default)]
#[builder(derive(Debug))]
#[builder(build_fn(error = "OpenAIError"))]
pub struct ChatCompletionMessage {
    pub role: Role,
    pub content: String,
    pub name: Option<String>,
}

#[derive(Builder, Clone, Debug, Default, Serialize)]
#[builder(name = "CreateChatRequestBuilder")]
#[builder(pattern = "mutable")]
#[builder(setter(into, strip_option), default)]
#[builder(derive(Debug))]
#[builder(build_fn(error = "OpenAIError"))]
pub struct CreateChatRequest {
    pub model: String,
    pub messages: Vec<ChatCompletionMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ChatChoice {
    pub message: ChatCompletionMessage,
    pub finish_reason: String,
    pub index: u32,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: u32,
    pub choices: Vec<ChatChoice>,
    pub usage: ChatUsage,
}
pub struct Chat {
    client: ClientEnum,
}

impl Chat {
    pub fn new(client: ClientEnum) -> Self {
        Self { client }
    }

    pub async fn create(&self, req: &CreateChatRequest) -> OpenAIResponse<ChatResponse> {
        self.client.post("/chat/completions", req).await
    }
    pub async fn create_with_template(
        &self,
        template: PromptTemplate,
        objects: HashMap<String, String>,
        model: &str,
    ) -> OpenAIResponse<ChatResponse> {
        let prompt = template.generate_prompt(objects);

        let message = ChatCompletionMessage {
            role: Role::User,
            content: prompt,
            name: None,
        };

        let req = CreateChatRequestBuilder::default()
            .model(model)
            .messages(vec![message])
            .build()
            .map_err(|_| {
                OpenAIError::InvalidArgument("Failed to build CreateChatRequest".into())
            })?;

        self.create(&req).await
    }
}
