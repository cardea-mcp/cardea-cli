use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
};
use reqwest::Client;
use rmcp::transport::sse_server::MiddlewareFn;
use serde::Serialize;
use serde_json::json;

#[derive(Clone)]
pub struct ApiKeyVerifier {
    pub client: Client,
    pub base_url: String,
}

impl ApiKeyVerifier {
    pub fn new(base_url: String) -> Self {
        ApiKeyVerifier {
            client: Client::new(),
            base_url,
        }
    }

    async fn verify_api_key(&self, api_key: &str) -> Result<(), String> {
        let url: String = format!("{}/reverse-lookup", self.base_url);
        let req_body = ReverseLookupRequest {
            api_key: api_key.to_string(),
        };

        let res = self
            .client
            .post(url)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(format!("Invalid API key (HTTP {})", res.status()))
        }
    }
}

#[derive(Serialize)]
struct ReverseLookupRequest {
    api_key: String,
}

pub async fn key_verify_layer(
    State(verifier): State<ApiKeyVerifier>,
    req: Request<Body>,
    next: Next,
) -> Response {
    match req.headers().get("Authorization") {
        Some(header_value) => match header_value.to_str() {
            Ok(auth_header) => {
                if let Some(key) = auth_header.strip_prefix("Bearer ") {
                    match verifier.verify_api_key(key).await {
                        Ok(_) => next.run(req).await,
                        Err(msg) => {
                            let body = json!({ "code": 401, "message": msg });
                            (StatusCode::UNAUTHORIZED, Json(body)).into_response()
                        }
                    }
                } else {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "code": 400, "message": "Invalid Authorization format" })),
                    )
                        .into_response()
                }
            }
            Err(_) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "code": 400, "message": "Invalid Authorization header encoding" })),
            )
                .into_response(),
        },
        None => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "code": 401, "message": "Missing Authorization header" })),
        )
            .into_response(),
    }
}

pub fn auth_middleware(base_url: String) -> MiddlewareFn {
    let verifier = ApiKeyVerifier::new(base_url);
    Box::new(move |router: Router| {
        router.route_layer(from_fn_with_state(verifier.clone(), key_verify_layer))
    })
}
