use dotenvy::dotenv;
use std::net::SocketAddr;

mod agent;
mod api;
mod http;
mod model;

#[tokio::main]
async fn main() {
    // load environment variables from .env file
    dotenv().ok();
    // initialize tracing
    tracing_subscriber::fmt::init();

    let app = api::router();

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
