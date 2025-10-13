use std::net::SocketAddr;
use axum::{self,routing::{get,post}};
use tower_http;
use tokio;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() 
{
    let port = 3334;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let page_router = axum::Router::new()
        .route("/hello", get( || async {"hello"}) )
        .nest_service("/",ServeDir::new("frontend/dist"))
        .into_make_service();
    let _server = hyper::Server::bind(&addr).serve(page_router).await.unwrap();
}
