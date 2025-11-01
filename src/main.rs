use anyhow::{ensure, Result};
use axum::{self, Json, debug_handler, extract::State, response::IntoResponse, routing::{delete, get, post, put}};
use lib::*;
use rusqlite::{self, Connection, OptionalExtension};
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc};
use tokio::{self, sync::Mutex};
use tower_http;
use tower_http::services::ServeDir;
type AppState = Arc<Mutex<rusqlite::Connection>>;

#[cfg(debug_assertions)]
const BIND: [u8; 4] = [127, 0, 0, 1];
#[cfg(debug_assertions)]
const PORT: u16 = 3334;
// release build will be out-facing
#[cfg(not(debug_assertions))]
const BIND: [u8; 4] = [0, 0, 0, 0];
#[cfg(not(debug_assertions))]
const PORT: u16 = 80; // or 443 for https if we cert up

#[tokio::main]
async fn main()
{
    let addr = SocketAddr::from((BIND, PORT)); //0.0.0.0 for outfacing
    let db = init_db().expect("Couldn't connect to database");
    let state = Arc::new(Mutex::new(db));
    let api_router = axum::Router::new().route("/kv_set", put(kv_set))
                                        .route("/kv_get", post(kv_get))
                                        .route("/kv_append", post(kv_append))
                                        .route("/kv_delete", delete(kv_delete))
                                        .with_state(state);
    let page_router = axum::Router::new().route("/hello", get(|| async { "hello" }))
                                         .nest_service("/api", api_router)
                                         .nest_service("/", ServeDir::new("frontend/dist"))
                                         .into_make_service();
    let _server = hyper::Server::bind(&addr).serve(page_router).await.unwrap();
}
#[derive(Deserialize)]
struct KVSetRequest
{
    key:   String,
    value: String,
}

#[debug_handler]
async fn kv_set(state: State<AppState>, req: Json<KVSetRequest>) -> Result<impl IntoResponse, String>
{
    let cnxn = state.lock().await;
    db_set(&cnxn, &req.key, &req.value).map_err(|e| e.to_string())?;
    return Ok(());
}

#[derive(Deserialize)]
struct KVGetRequest
{
    key: String,
}

#[debug_handler]
async fn kv_get(state: State<AppState>, req: Json<KVGetRequest>) -> Result<impl IntoResponse, String>
{
    let cnxn = state.lock().await;
    db_get(&cnxn, &req.key).map_err(|e| e.to_string())?;
    return Ok(());
}

#[debug_handler]
async fn kv_append(state: State<AppState>, req: Json<KVSetRequest>) -> Result<impl IntoResponse, String>
{
    let cnxn = state.lock().await;
    db_append(&cnxn, &req.key, &req.value).map_err(|e| e.to_string())?;
    return Ok(());
}

async fn kv_delete(state: State<AppState>, req: Json<KVGetRequest>) -> Result<impl IntoResponse, String>
{
    let cnxn = state.lock().await;
    db_delete(&cnxn, &req.key).map_err(|e| e.to_string())?;
    return Ok(());
}