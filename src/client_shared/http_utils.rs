use std::marker::PhantomData;
use std::sync::Arc;

#[cfg(feature = "async-compat")]
use async_compat::Compat;
use reqwest::{RequestBuilder, Response, StatusCode};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use snafu::{whatever, ResultExt, Whatever};

use crate::client_next::correlation::{CorrelationConfig, CorrelationData};

// use super::errors::{registration_error, RegistrationError};


// Serde objects

/// Serde struct used to deserialize the
/// json body of a poll response
#[derive(Deserialize)]
pub(crate) struct PollResponse {
    pub(crate) aes_key: String,

    #[serde(rename(deserialize = "data"))]
    pub(crate) data_list: Option<Vec<String>>,
}


/// Serde struct used to serialize the body data
/// for a deregister post request
#[derive(Serialize)]
pub struct DeregisterData {
    #[serde(rename(serialize = "correlation-id"))]
    pub(crate) correlation_id: String,

    #[serde(rename(serialize = "secret-key"))]
    pub(crate) secret_key: String,
}


/// Serde struct used to serialize the body data
/// for a register post request
#[derive(Serialize)]
pub struct RegisterData {
    #[serde(rename(serialize = "public-key"))]
    pub(crate) public_key: String,

    #[serde(rename(serialize = "secret-key"))]
    pub(crate) secret_key: String,

    #[serde(rename(serialize = "correlation-id"))]
    pub(crate) correlation_id: String,
}


/// Unit struct used only as an unused generic placeholder
#[derive(Serialize)]
pub struct EmptyData;

pub enum HttpRequest<P: Serialize + Send> {
    Get {
        url: String,
        query_params: SmallVec<[(String, String); 2]>,
        phantom_data: PhantomData<P>,
    },
    Post {
        url: String,
        post_data: P,
    },
}

impl HttpRequest<EmptyData> {
    pub fn new_get_request(
        url: String,
        query_params: SmallVec<[(String, String); 2]>,
    ) -> HttpRequest<EmptyData> {
        let phantom_data: PhantomData<EmptyData> = PhantomData;

        Self::Get {
            url,
            query_params,
            phantom_data,
        }
    }
}

impl<P: Serialize + Send> HttpRequest<P> {
    fn create_request_builder(&self, reqwest_client: &reqwest::Client) -> RequestBuilder {
        match self {
            HttpRequest::Get {
                url, query_params, ..
            } => reqwest_client.get(url).query(query_params),
            HttpRequest::Post { url, post_data } => reqwest_client.post(url).json(&post_data),
        }
    }
}

pub async fn make_http_request<P: Serialize + Send>(
    reqwest_client: &reqwest::Client,
    auth_token: Option<&Secret<String>>,
    request_info: HttpRequest<P>,
) -> Result<Response, reqwest::Error> {
    let mut http_request = request_info.create_request_builder(reqwest_client);

    http_request = match auth_token {
        Some(token) => http_request.header("Authorization", token.expose_secret()),
        None => http_request,
    };

    cfg_if::cfg_if! {
        if #[cfg(feature = "async-compat")] {
            let http_request_future = Compat::new(async {
                http_request.send().await
            });
        } else {
            let http_request_future = http_request.send();
        }
    }

    http_request_future.await
}


#[derive(PartialEq, Eq)]
pub enum ClientStatus {
    Unregistered,
    Registered {
        subdomain: String,
        correlation_id: String,
    },
}

enum RegistrationAction {
    Register,
    Deregister,
}

impl RegistrationAction {
    fn into_action_url(self, server_name: &str) -> String {
        let action = match self {
            RegistrationAction::Register => "register",
            RegistrationAction::Deregister => "deregister",
        };

        format!("https://{server_name}/{action}")
    }
}

pub struct ServerComm {
    pub(crate) server_name: String,
    pub(crate) auth_token: Option<Secret<String>>,
    pub(crate) secret_key: Secret<String>,
    pub(crate) encoded_pub_key: String,
    pub(crate) reqwest_client: Arc<reqwest::Client>,
    pub(crate) correlation_config: Option<CorrelationConfig>,
    pub(crate) status: ClientStatus,
}

impl ServerComm {
    pub(crate) fn get_interaction_fqdn(&self) -> Option<String> {
        match &self.status {
            ClientStatus::Unregistered => None,
            ClientStatus::Registered { subdomain, .. } => {
                Some(format!("{}.{}", subdomain, self.server_name))
            }
        }
    }

    pub(crate) async fn register(&mut self) -> Result<(), Whatever> {
        if let ClientStatus::Registered { .. } = self.status {
            whatever!("Already registered");
        }

        let correlation_data = match &self.correlation_config {
            Some(config) => CorrelationData::generate_data(config),
            None => CorrelationData::default(),
        };

        let post_data = RegisterData {
            public_key: self.encoded_pub_key.clone(),
            secret_key: self.secret_key.expose_secret().clone(),
            correlation_id: correlation_data.correlation_id.clone(),
        };

        self.perform_registration_action(RegistrationAction::Register, post_data)
            .await?;
        self.status = correlation_data.into();

        Ok(())
    }

    pub(crate) async fn deregister(&mut self) -> Result<(), Whatever> {
        let correlation_id = match &self.status {
            ClientStatus::Unregistered => whatever!("Not currently registered"),
            ClientStatus::Registered { correlation_id, .. } => correlation_id.clone(),
        };

        let post_data = DeregisterData {
            correlation_id,
            secret_key: self.secret_key.expose_secret().clone(),
        };

        self.perform_registration_action(RegistrationAction::Deregister, post_data)
            .await?;
        self.status = ClientStatus::Unregistered;

        Ok(())
    }

    pub(crate) async fn force_deregister(&mut self) {
        self.deregister().await.ok();
        self.status = ClientStatus::Unregistered;
    }

    pub(crate) async fn poll(&self) -> Result<PollResponse, Whatever> {
        let correlation_id = match &self.status {
            ClientStatus::Unregistered => whatever!("Not currently registered"),
            ClientStatus::Registered { correlation_id, .. } => correlation_id.clone(),
        };
        let poll_url = format!("https://{server_name}/poll", server_name = self.server_name);

        let mut query_params = SmallVec::<[(String, String); 2]>::new();
        query_params.push(("id".into(), correlation_id));
        query_params.push(("secret".into(), self.secret_key.expose_secret().clone()));

        let request_info = HttpRequest::new_get_request(poll_url, query_params);

        let get_response =
            make_http_request(&self.reqwest_client, self.auth_token.as_ref(), request_info)
                .await
                .whatever_context("Poll failed")?;

        let response_status = get_response.status();

        if response_status.is_success() {
            let response_body = get_response
                .json::<PollResponse>()
                .await
                .whatever_context("Failed to parse response body")?;

            Ok(response_body)
        } else {
            let server_msg = get_response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let status_code = response_status.as_u16();
            whatever!("Poll failed: {} - {}", status_code, server_msg)
        }
    }

    async fn perform_registration_action<P: Serialize + Send>(
        &mut self,
        action: RegistrationAction,
        post_data: P,
    ) -> Result<(), Whatever> {
        let action_url = action.into_action_url(&self.server_name);
        let request_info = HttpRequest::Post {
            url: action_url,
            post_data,
        };

        let register_response =
            make_http_request(&self.reqwest_client, self.auth_token.as_ref(), request_info)
                .await
                .whatever_context("Failed to send request")?;

        let response_status = register_response.status();
        if response_status.is_success() {
            Ok(())
        } else if response_status == StatusCode::UNAUTHORIZED {
            whatever!("Unauthorized")
        } else {
            let server_msg = register_response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let status_code = response_status.as_u16();
            whatever!("Registration failed: {} - {}", status_code, server_msg)
        }
    }
}
