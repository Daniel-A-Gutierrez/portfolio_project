use anyhow::{anyhow, Result};
use axum::{self, debug_handler,
           extract::{Json, State},
           response::IntoResponse,
           routing::{delete, get, post, put}};
use lib::{db_get, db_set, init_db, KVRow, SetError};
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc};
use tokio::{self, sync::Mutex};
type AppState = Arc<Mutex<rusqlite::Connection>>;

#[tokio::main]
async fn main()
{
    let addr = SocketAddr::from(([127, 0, 0, 1], 3334)); //0.0.0.0 for outfacing
    let db = init_db().expect("Couldn't connect to database");
    let state = Arc::new(Mutex::new(db));
    let api_router = axum::Router::new().route("/hello", get(hello))
                                        .route("/kv_get", post(kv_get))
                                        .route("/kv_set", put(kv_set))
                                        .with_state(state)
                                        .into_make_service();
    let _server = hyper::Server::bind(&addr).serve(api_router).await.unwrap();
}

#[debug_handler]
async fn hello() -> impl IntoResponse
{
    return "hello";
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
