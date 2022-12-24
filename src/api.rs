use crate::model::ScrapeRequestBody;
use crate::{agent::Agent, http};
use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use hyper::HeaderMap;

#[derive(Debug, Clone)]
struct AppState {
    agents: Vec<Agent>,
    api_key: String,
}

pub fn router() -> Router {
    let state = AppState {
        agents: Agent::from_env(),
        api_key: std::env::var("API_KEY").expect("ENV 'API_KEY' NOT FOUND!"),
    };

    Agent::restart_interval(state.agents.clone());

    let api = Router::new()
        .route("/scrape", post(scrape))
        .route("/scrape-js", post(scrape_js))
        .with_state(state);

    Router::new().nest("/api/v1", api)
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
    let agent = Agent::get_available(state.agents).await?;

    let path = if js_enabled { "scrape-js" } else { "scrape" };
    let url = format!("{}/{}", agent.url, path);
    tracing::info!("calling agent url: {}", url);

    let body = http::hyper_post(&url, &payload).await?;

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
