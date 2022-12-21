use crate::model::ScrapeRequestBody;
use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use hyper::{Body, Client, HeaderMap, Method, Request};

#[derive(Clone)]
struct AppState {
    agent_url: String,
    api_key: String,
}

pub fn router() -> Router {
    let state = AppState {
        agent_url: std::env::var("AGENT_URL").expect("ENV 'AGENT_URL' NOT FOUND!"),
        api_key: std::env::var("API_KEY").expect("ENV 'API_KEY' NOT FOUND!"),
    };

    let api = Router::new()
        .route("/scrape", post(scrape))
        .route("/scrape-js", post(scrape_js))
        .with_state(state);

    Router::new().nest("/api/v1", api)
}

async fn hyper_post(url: &str, request_body: &ScrapeRequestBody) -> anyhow::Result<String> {
    let client = Client::new();
    let req = Request::builder()
        .method(Method::POST)
        .uri(url)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(request_body)?))?;

    let resp = client.request(req).await?;
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
    let out = String::from_utf8(body_bytes.to_vec())?;

    Ok(out)
}

//TODO: change this to a middleware
fn check_api_key(state: &AppState, headers: HeaderMap) -> anyhow::Result<()> {
    let api_key = headers
        .get("API_KEY")
        .ok_or_else(|| anyhow!("API_KEY NOT FOUND"))?
        .to_str()?;
    if api_key != state.api_key {
        return Err(anyhow!("API_KEY IS INVALID"));
    }
    Ok(())
}

async fn scrape_template(
    headers: HeaderMap,
    state: AppState,
    payload: ScrapeRequestBody,
    js_enabled: bool,
) -> anyhow::Result<(StatusCode, String)> {
    check_api_key(&state, headers)?;

    let path = if js_enabled { "scrape-js" } else { "scrape" };
    let url = format!("{}/{}", state.agent_url, path);
    tracing::info!("calling agent url: {}", url);

    let body = hyper_post(&url, &payload).await?;

    Ok((StatusCode::OK, body))
}

async fn scrape_js(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<ScrapeRequestBody>,
) -> impl IntoResponse {
    match scrape_template(headers, state, payload, true).await {
        Ok((status, body)) => (status, body),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn scrape(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<ScrapeRequestBody>,
) -> impl IntoResponse {
    match scrape_template(headers, state, payload, false).await {
        Ok((status, body)) => (status, body),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}
