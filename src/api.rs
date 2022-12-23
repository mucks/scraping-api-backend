use crate::model::ScrapeRequestBody;
use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use hyper::{Body, Client, HeaderMap, Method, Request};

#[derive(Debug, Clone)]
struct Agent {
    id: u32,
    url: String,
}

#[derive(Debug, Clone)]
struct AppState {
    agents: Vec<Agent>,
    api_key: String,
}

pub fn router() -> Router {
    let agent_urls = std::env::var("AGENT_URLS").expect("ENV 'AGENT_URLS' NOT FOUND!");
    let agent_replicas: Vec<(&str, u32)> = agent_urls
        .split(',')
        .filter_map(|f| f.split_once("|replicas="))
        .map(|(url, replicas)| (url, replicas.parse().unwrap_or(1)))
        .collect();

    let mut agents: Vec<Agent> = vec![];
    for agent_replica in agent_replicas {
        for i in 0..agent_replica.1 {
            agents.push(Agent {
                id: i,
                url: format!("{}-{}", agent_replica.0, i),
            });
        }
    }

    let state = AppState {
        agents,
        api_key: std::env::var("API_KEY").expect("ENV 'API_KEY' NOT FOUND!"),
    };

    let api = Router::new()
        .route("/scrape", post(scrape))
        .route("/scrape-js", post(scrape_js))
        .with_state(state);

    Router::new().nest("/api/v1", api)
}

async fn hyper_get(url: &str) -> anyhow::Result<String> {
    let client = Client::new();
    let resp = client.get(url.parse()?).await?;
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
    let out = String::from_utf8(body_bytes.to_vec())?;
    Ok(out)
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

async fn get_available_agent(state: &AppState) -> anyhow::Result<&Agent> {
    let mut available_agents: Vec<&Agent> = vec![];

    for agent in state.agents.iter() {
        let is_busy = hyper_get(&format!("{}/is-busy", agent.url)).await?;
        if is_busy == "false" {
            available_agents.push(agent);
        }
    }

    if available_agents.is_empty() {
        return Err(anyhow!("NO AVAILABLE AGENTS"));
    }

    let index = rand::random::<usize>() % available_agents.len();
    let agent = available_agents[index];

    Ok(agent)
}

async fn scrape_template(
    headers: HeaderMap,
    state: AppState,
    payload: ScrapeRequestBody,
    js_enabled: bool,
) -> anyhow::Result<(StatusCode, String)> {
    check_api_key(&state, headers)?;
    let agent = get_available_agent(&state).await?;

    let path = if js_enabled { "scrape-js" } else { "scrape" };
    let url = format!("{}-{}/{}", agent.url, agent.id, path);
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
