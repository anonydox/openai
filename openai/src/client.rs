use crate::core::response_wrapper::{ApiErrorResponse, OpenAIError};
use async_trait::async_trait;
use reqwest::{header::HeaderMap, Client, Method, RequestBuilder};

use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

pub const ORGANIZATION_HEADER: &str = "OpenAI-Organization";

#[async_trait]
pub trait ReqClient: Sync + Send {
    fn headers(&self) -> HeaderMap;

    fn api_key(&self) -> &str;

    fn api_base(&self) -> String;

    async fn get<T, F>(&self, route: &str, query: &F) -> Result<T, OpenAIError>
    where
        T: DeserializeOwned + Debug + Send,
        F: Serialize + Send + Sync;

    async fn post<T, F>(&self, route: &str, json: &F) -> Result<T, OpenAIError>
    where
        T: DeserializeOwned + Debug + Send,
        F: Serialize + Send + Sync;
}

async fn resolve_response<T>(request: RequestBuilder) -> Result<T, OpenAIError>
where
    T: DeserializeOwned + Debug + Send,
{
    let response = request.send().await?;
    let status = response.status();
    let bytes = response.bytes().await?;
    if !status.is_success() {
        let api_error: ApiErrorResponse =
            serde_json::from_slice(bytes.as_ref()).map_err(OpenAIError::JSONDeserialize)?;
        return Err(OpenAIError::ApiError(api_error));
    }
    let data: T = serde_json::from_slice(bytes.as_ref()).map_err(OpenAIError::JSONDeserialize)?;
    Ok(data)
}

pub struct ClientBase {
    pub api_key: String,
    pub base_url: String,
}

impl ClientBase {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self { api_key, base_url }
    }

    fn headers(&self) -> HeaderMap {
        HeaderMap::new()
    }

    fn api_key(&self) -> &str {
        &self.api_key
    }

    fn api_base(&self) -> &str {
        &self.base_url
    }

    fn request<F>(&self, method: Method, route: &str, builder: F) -> RequestBuilder
    where
        F: FnOnce(RequestBuilder) -> RequestBuilder + Send,
    {
        let client = Client::new();
        let mut request = client
            .request(method, format!("{}{}", self.api_base(), route))
            .headers(self.headers())
            .bearer_auth(self.api_key());
        request = builder(request);
        request
    }

    async fn get<T, F>(&self, route: &str, query: &F) -> Result<T, OpenAIError>
    where
        T: DeserializeOwned + Debug + Send,
        F: Serialize + Send + Sync,
    {
        let request = self.request(Method::GET, route, |req| req.query(query));
        resolve_response(request).await
    }

    async fn post<T, F>(&self, route: &str, json: &F) -> Result<T, OpenAIError>
    where
        T: DeserializeOwned + Debug + Send,
        F: Serialize + Send + Sync,
    {
        let request = self.request(Method::POST, route, |req| req.json(json));
        resolve_response(request).await
    }
}

pub struct OpenAI {
    base: ClientBase,
    pub org_id: Option<String>,
}

impl OpenAI {
    pub fn new(api_key: String, org_id: Option<String>) -> Self {
        let base_url = "https://api.openai.com/v1".to_string();
        Self {
            base: ClientBase::new(api_key, base_url),
            org_id,
        }
    }
}

#[async_trait]
impl ReqClient for OpenAI {
    fn headers(&self) -> HeaderMap {
        let mut headers = self.base.headers();
        if let Some(org_id) = &self.org_id {
            headers.insert(ORGANIZATION_HEADER, org_id.parse().unwrap());
        }
        headers
    }

    fn api_key(&self) -> &str {
        self.base.api_key()
    }

    fn api_base(&self) -> String {
        self.base.api_base().to_string()
    }

    async fn get<T, F>(&self, route: &str, query: &F) -> Result<T, OpenAIError>
    where
        T: DeserializeOwned + Debug + Send,
        F: Serialize + Send + Sync,
    {
        self.base.get(route, query).await
    }

    async fn post<T, F>(&self, route: &str, json: &F) -> Result<T, OpenAIError>
    where
        T: DeserializeOwned + Debug + Send,
        F: Serialize + Send + Sync,
    {
        self.base.post(route, json).await
    }
}
pub struct AzureOpenAI {
    base: ClientBase,
    pub resource_name: String,
    pub deployment_id: String,
    pub api_version: String,
}

impl AzureOpenAI {
    pub fn new(
        api_key: String,
        resource_name: String,
        deployment_id: String,
        api_version: String,
    ) -> Self {
        let base_url = format!("https://{}.openai.azure.com", resource_name);
        Self {
            base: ClientBase::new(api_key, base_url),
            resource_name,
            deployment_id,
            api_version,
        }
    }

    fn route_with_deployment(&self, route: &str) -> String {
        format!(
            "/openai/deployments/{}/{}?api-version={}",
            self.deployment_id, route, self.api_version
        )
    }
}

#[async_trait]
impl ReqClient for AzureOpenAI {
    fn headers(&self) -> HeaderMap {
        self.base.headers()
    }

    fn api_key(&self) -> &str {
        self.base.api_key()
    }

    fn api_base(&self) -> String {
        self.base.api_base().to_string()
    }

    async fn get<T, F>(&self, route: &str, query: &F) -> Result<T, OpenAIError>
    where
        T: DeserializeOwned + Debug + Send,
        F: Serialize + Send + Sync,
    {
        let route = self.route_with_deployment(route);
        self.base.get(&route, query).await
    }

    async fn post<T, F>(&self, route: &str, json: &F) -> Result<T, OpenAIError>
    where
        T: DeserializeOwned + Debug + Send,
        F: Serialize + Send + Sync,
    {
        let route = self.route_with_deployment(route);
        self.base.post(&route, json).await
    }
}

pub enum ClientEnum {
    OpenAI(OpenAI),
    AzureOpenAI(AzureOpenAI),
}

impl ClientEnum {
    pub fn headers(&self) -> HeaderMap {
        match self {
            ClientEnum::OpenAI(client) => client.headers(),
            ClientEnum::AzureOpenAI(client) => client.headers(),
        }
    }

    pub fn api_key(&self) -> &str {
        match self {
            ClientEnum::OpenAI(client) => client.api_key(),
            ClientEnum::AzureOpenAI(client) => client.api_key(),
        }
    }

    pub fn api_base(&self) -> String {
        match self {
            ClientEnum::OpenAI(client) => client.api_base(),
            ClientEnum::AzureOpenAI(client) => client.api_base(),
        }
    }

    pub async fn get<T, F>(&self, route: &str, query: &F) -> Result<T, OpenAIError>
    where
        T: DeserializeOwned + Debug + Send,
        F: Serialize + Send + Sync,
    {
        match self {
            ClientEnum::OpenAI(client) => client.get(route, query).await,
            ClientEnum::AzureOpenAI(client) => client.get(route, query).await,
        }
    }

    pub async fn post<T, F>(&self, route: &str, json: &F) -> Result<T, OpenAIError>
    where
        T: DeserializeOwned + Debug + Send,
        F: Serialize + Send + Sync,
    {
        match self {
            ClientEnum::OpenAI(client) => client.post(route, json).await,
            ClientEnum::AzureOpenAI(client) => client.post(route, json).await,
        }
    }
}
