use actix_web::{web, HttpRequest, HttpResponse, Error};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{services, auth};

// Structures for OpenAI compatible requests
#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: Option<bool>,
    temperature: Option<f32>,
}

// Structures for the target Atlassian AI API
#[derive(Serialize)]
struct AtlassianRequestPayload {
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Serialize)]
struct AtlassianPlatformAttrs {
    model: String,
}

#[derive(Serialize)]
struct AtlassianRequest {
    #[serde(rename = "requestPayload")]
    request_payload: AtlassianRequestPayload,
    #[serde(rename = "platformAttributes")]
    platform_attributes: AtlassianPlatformAttrs,
}

// Health check handler
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(json!({ "status": "ok" }))
}

// Mock models handler (as before)
pub async fn list_models() -> HttpResponse {
    let response = json!({ "object": "list", "data": [ { "id": "gpt-3.5-turbo", "object": "model", "owned_by": "system" } ] });
    HttpResponse::Ok().json(response)
}

// The core chat completions proxy handler
pub async fn chat_completions(
    req: HttpRequest,
    body: web::Json<ChatCompletionRequest>,
    client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    // 1. Validate API Token from Authorization header
    let auth_header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());
    if auth_header.is_none() || !auth_header.unwrap().starts_with("Bearer ") {
        return Ok(HttpResponse::Unauthorized().json(json!({ "error": "API key is required" })));
    }
    let token = auth_header.unwrap().strip_prefix("Bearer ").unwrap();
    if services::validate_api_token(token).is_err() {
        return Ok(HttpResponse::Unauthorized().json(json!({ "error": "Invalid API key" })));
    }

    // 2. Get all available credentials from the database
    let credentials = match services::list_credentials() {
        Ok(creds) if !creds.is_empty() => creds,
        _ => return Ok(HttpResponse::InternalServerError().json(json!({ "error": "No credentials configured" }))),
    };

    // 3. Prepare the request for the target service
    let atlassian_req = AtlassianRequest {
        request_payload: AtlassianRequestPayload {
            messages: body.messages.clone(),
            temperature: body.temperature,
            stream: body.stream.unwrap_or(false),
        },
        platform_attributes: AtlassianPlatformAttrs {
            model: body.model.clone(), // We can add model mapping logic here later
        },
    };

    // 4. Loop through credentials and attempt to make a request
    for credential in credentials {
        let request_builder = client
            .post("https://api.atlassian.com/ai/chat/completions") // Target URL
            .bearer_auth(&credential.token)
            .json(&atlassian_req);

        match request_builder.send().await {
            Ok(response) if response.status().is_success() => {
                // 5. Handle successful response (streaming or non-streaming)
                if body.stream.unwrap_or(false) {
                    let stream = response.bytes_stream().map(|item| item.map_err(actix_web::error::ErrorInternalServerError));
                    return Ok(HttpResponse::Ok()
                        .content_type("text/event-stream")
                        .streaming(stream));
                } else {
                    let response_body = response.json::<serde_json::Value>().await.unwrap();
                    return Ok(HttpResponse::Ok().json(response_body));
                }
            }
            Ok(failed_response) => {
                // Log error and try next credential
                log::warn!("Credential for {} failed with status: {}", credential.email, failed_response.status());
                continue;
            }
            Err(e) => {
                // Log error and try next credential
                log::error!("Request with credential for {} failed: {}", credential.email, e);
                continue;
            }
        }
    }

    // If all credentials failed
    Ok(HttpResponse::BadGateway().json(json!({ "error": "All credentials exhausted" })))
}
