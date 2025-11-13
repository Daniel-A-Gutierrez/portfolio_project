use anyhow::{ensure, Result};
use axum::{self, debug_handler,
           extract::State,
           http::StatusCode,
           middleware::{self, from_fn, from_fn_with_state, Next},
           response::IntoResponse,
           routing::{delete, get, post, put},
           Json};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use dashmap::DashSet;
use hyper::{Body, Request};
use lib::*;
use rand::prelude::*;
use rusqlite::{self, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::{self, sync::Mutex};
use tower_http;
use tower_http::services::ServeDir;

struct S
{
    pub db:       Mutex<rusqlite::Connection>,
    pub sessions: DashSet<u128>,
}

type AppState = Arc<S>;

#[cfg(debug_assertions)]
const BIND: [u8; 4] = [127, 0, 0, 1];
#[cfg(debug_assertions)]
const PORT: u16 = 3334;
// release build will be out-facing
#[cfg(not(debug_assertions))]
const BIND: [u8; 4] = [0, 0, 0, 0];
#[cfg(not(debug_assertions))]
const PORT: u16 = 80; // or 443 for https if we cert up

#[tokio::main(flavor = "current_thread")]
async fn main()
{
    let addr = SocketAddr::from((BIND, PORT)); //0.0.0.0 for outfacing
    let db = init_db().expect("Couldn't connect to database");
    let state = Arc::new(S { db:       Mutex::new(db),
                             sessions: DashSet::new(), });
    let api_router = axum::Router::new().route("/kv_set", put(kv_set))
                                        .route("/kv_get", post(kv_get))
                                        .route("/kv_json_append", post(kv_json_append))
                                        .route("/kv_delete", delete(kv_delete))
                                        .layer(from_fn_with_state(state.clone(), check_session))
                                        .route("/get_session", post(get_session))
                                        .with_state(state);
    let page_router = axum::Router::new().route("/hello", get(|| async { "hello" }))
                                         .nest_service("/api", api_router)
                                         .nest_service("/", ServeDir::new("frontend/dist"))
                                         .into_make_service();
    let _server = hyper::Server::bind(&addr).serve(page_router).await.unwrap();
}

#[debug_handler]
async fn kv_set(state: State<AppState>, req: Json<KVSetRequest>) -> Result<impl IntoResponse, String>
{
    let cnxn = &state.db.lock().await;
    db_set(cnxn, &req.key, &req.value).map_err(|e| e.to_string())?;
    return Ok(());
}

#[debug_handler]
async fn kv_get(state: State<AppState>, req: Json<KVGetRequest>) -> Result<impl IntoResponse, String>
{
    let cnxn = &state.db.lock().await;
    let entry = db_get(cnxn, &req.key).map_err(|e| e.to_string())?;
    return Ok(Json(entry));
}

#[debug_handler]
async fn kv_json_append(state: State<AppState>, req: Json<KVSetRequest>)
                        -> Result<impl IntoResponse, String>
{
    let cnxn = &state.db.lock().await;
    db_json_append(cnxn, &req.key, &req.value).map_err(|e| e.to_string())?;
    return Ok(());
}

#[debug_handler]
async fn kv_delete(state: State<AppState>, req: Json<KVGetRequest>) -> Result<impl IntoResponse, String>
{
    let cnxn = &state.db.lock().await;
    db_delete(cnxn, &req.key).map_err(|e| e.to_string())?;
    return Ok(());
}

#[debug_handler]
async fn get_session(state: State<AppState>,
                     mut jar: CookieJar,
                     req: Json<SessionRequest>)
                     -> Result<impl IntoResponse, String>
{
    //if theres already a session cookie and its in the map, delete it from the map.
    if let Some(old_cookie) = jar.get("session_id")
    {
        let old_cookie: u128 =
            u128::from_str_radix(old_cookie.value(), 16).map_err(|_| "Session ID must be hexadecimal")?;
        state.sessions.remove(&old_cookie);
    }

    if req.answer == "please"
    {
        let new_session: u128 = rand::rng().random::<u128>();
        jar = jar.add(Cookie::new("session_id", format!("{:X}", new_session)));
        state.sessions.insert(new_session); //this is why you dont program tired.
        return Ok((StatusCode::OK, jar));
    }
    return Ok((StatusCode::FORBIDDEN, jar));
}

async fn check_session(state: State<AppState>,
                       jar: CookieJar,
                       req: Request<Body>,
                       next: Next<Body>)
                       -> impl IntoResponse
{
    println!("{}", "checking session");
    if let Some(cookie) = jar.get("session_id")
    {
        println!("{:?}", cookie);
        println!("{:?}", state.sessions);
        if let Ok(session_id) = u128::from_str_radix(cookie.value(), 16)
        {
            if state.sessions.contains(&session_id)
            {
                return next.run(req).await.into_response();
            }
        }
        else
        {
            return (StatusCode::UNAUTHORIZED, jar, "Session ID must be hexadecimal").into_response();
        }
    }
    //its testy about just giving back the status code.
    return (StatusCode::UNAUTHORIZED, jar, "").into_response();
}
