use axum::{
    extract::{Query, State, Form},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router
};
use serde::{Deserialize};
use std::{net::SocketAddr, sync::Arc};
use tokio;

#[derive(Clone)]
struct Config {
    authentik_url: String,
}

#[derive(Deserialize)]
struct AuthorizeQuery {
    client_id: String,
    redirect_uri: String,
    response_type: String,
    state: Option<String>,
    scope: Option<String>,
}

#[derive(Deserialize)]
struct TokenForm {
    client_id: String,
    client_secret: String,
    code: String,
    redirect_uri: String,
    grant_type: String,
}

async fn oauth_authorize(
    State(config): State<Arc<Config>>,
    Query(q): Query<AuthorizeQuery>,
) -> impl IntoResponse {
    // Redirect to Authentik's authorize endpoint, passing all params
    let mut url = format!(
        "{}/authorize?client_id={}&redirect_uri={}&response_type={}",
        config.authentik_url, q.client_id, q.redirect_uri, q.response_type
    );
    if let Some(state) = q.state {
        url.push_str(&format!("&state={}", state));
    }
    if let Some(scope) = q.scope {
        url.push_str(&format!("&scope={}", scope));
    }
    Redirect::temporary(&url)
}

async fn oauth_token(
    State(config): State<Arc<Config>>,
    Form(form): Form<TokenForm>,
) -> impl IntoResponse {
    // Just forward the code exchange to Authentik, return response as-is
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", form.grant_type.as_str()),
        ("code", form.code.as_str()),
        ("redirect_uri", form.redirect_uri.as_str()),
        ("client_id", form.client_id.as_str()),
        ("client_secret", form.client_secret.as_str()),
    ];
    let res = client
        .post(format!("{}/token", config.authentik_url))
        .form(&params)
        .send()
        .await;

    match res {
        Ok(resp) => {
            let status = axum::http::StatusCode::from_u16(resp.status().as_u16()).unwrap();
            let body = resp.bytes().await.unwrap_or_default();
            (status, body).into_response()
        }
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": "authentik_unreachable"})),
        )
            .into_response(),
    }
}

async fn api_v4_user(
    State(config): State<Arc<Config>>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<headers::Authorization<headers::authorization::Bearer>>,
) -> impl IntoResponse {
    // Forward /api/v4/user to Authentik's userinfo endpoint, return as-is (with GitLab schema conversion if needed)
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}/userinfo", config.authentik_url))
        .bearer_auth(headers.token())
        .send()
        .await;

    match res {
        Ok(resp) => {
            let status = axum::http::StatusCode::from_u16(resp.status().as_u16()).unwrap();
            let userinfo: serde_json::Value = match resp.json().await {
                Ok(json) => json,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "invalid_response"})),
                    )
                        .into_response();
                }
            };
            // Convert Authentik userinfo to GitLab's schema if needed
            let gitlab_user = serde_json::json!({
                "id": userinfo.get("sub").unwrap_or(&serde_json::Value::Null),
                "email": userinfo.get("email").unwrap_or(&serde_json::Value::Null),
                "name": userinfo.get("name").unwrap_or(&serde_json::Value::Null),
                "avatar_url": userinfo.get("avatar_url").unwrap_or(&serde_json::Value::Null),
                "family_name": userinfo.get("family_name").unwrap_or(&serde_json::Value::Null),
            });
            (status, Json(gitlab_user)).into_response()
        }
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": "authentik_unreachable"})),
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() {
    let config = Arc::new(Config {
        authentik_url: std::env::var("AUTHENTIK_URL").expect("AUTHENTIK_URL not set"),
    });

    let app = Router::new()
        .route("/oauth/authorize", get(oauth_authorize))
        .route("/oauth/token", post(oauth_token))
        .route("/api/v4/user", get(api_v4_user))
        .with_state(config);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Proxy running at {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}