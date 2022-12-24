use hyper::{Body, Client, Method, Request};

use crate::model::ScrapeRequestBody;

pub async fn hyper_get(url: &str) -> anyhow::Result<String> {
    let client = Client::new();
    let resp = client.get(url.parse()?).await?;
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
    let out = String::from_utf8(body_bytes.to_vec())?;
    Ok(out)
}

pub async fn hyper_post(url: &str, request_body: &ScrapeRequestBody) -> anyhow::Result<String> {
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

pub async fn hyper_delete(url: &str) -> anyhow::Result<()> {
    let client = Client::new();
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(url)
        .body(Body::empty())?;

    let _resp = client.request(req).await?;

    Ok(())
}
