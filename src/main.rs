use anyhow::{Result, ensure};
use axum::http::header;
use axum::{self, Json, Router, ServiceExt,
           body::Body,
           debug_handler,
           extract::{Request, State},
           handler::HandlerWithoutStateExt,
           http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
           middleware::{self, Next, from_fn, from_fn_with_state},
           response::IntoResponse,
           routing::{delete, get, post, put}};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use axum_response_cache::CacheLayer;
use axum_server::tls_rustls::RustlsConfig;
use dashmap::DashSet;
use lib::*;
use rand::prelude::*;
use rusqlite::{self, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{self, sync::Mutex};
use tower_http::compression::CompressionLayer;
use tower_http::decompression::DecompressionLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing::instrument::WithSubscriber;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
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
#[cfg(debug_assertions)]
const CLEAR_INTERVAL: u64 = 5000; //in millis
#[cfg(debug_assertions)]
const CACHE_LIFETIME: u64 = 1;
// release build will be out-facing
#[cfg(not(debug_assertions))]
const BIND: [u8; 4] = [0, 0, 0, 0];
#[cfg(not(debug_assertions))]
const PORT: u16 = 443; // or 443 for https if we cert up
#[cfg(not(debug_assertions))]
const CLEAR_INTERVAL: u64 = 1000 * 60 * 60;
#[cfg(not(debug_assertions))]
const CACHE_LIFETIME: u64 = 600;

#[tokio::main]
async fn main()
{
    let addr = SocketAddr::from((BIND, PORT)); //0.0.0.0 for outfacing
    let db = init_db().expect("Couldn't connect to database");
    let state = Arc::new(S { db:       Mutex::new(db),
                             sessions: DashSet::new(), });

    tracing_subscriber::registry().with(EnvFilter::try_from_default_env().or_else(|_| {
                                            EnvFilter::try_new(&format!("{}=warn,tower_http=debug",
                                                                        env!("CARGO_CRATE_NAME")))
                                        })
                                        .unwrap())
                                  .with(tracing_subscriber::fmt::layer())
                                  .init();

    let api_router = axum::Router::new().route("/kv_set", put(kv_set))
                                        .route("/kv_get", post(kv_get))
                                        .route("/kv_json_append", post(kv_json_append))
                                        .route("/kv_delete", delete(kv_delete))
                                        //.layer(from_fn_with_state(state.clone(), check_session))
                                        //.route("/get_session", post(get_session))
                                        .layer(CompressionLayer::new().quality(tower_http::CompressionLevel::Fastest))
                                        .with_state(state.clone());
    tokio::spawn(async move {
        loop
        {
            tokio::time::sleep(Duration::from_millis(CLEAR_INTERVAL)).await;
            let cnxn = state.db.lock().await;
            let _ = db_clear(&cnxn);
        }
    });
    let cache_control_layer =
        SetResponseHeaderLayer::appending(header::CACHE_CONTROL,
                                          HeaderValue::from_str(&format!("max-age:{},public",CACHE_LIFETIME)).unwrap());
    let page_router = axum::Router::new().fallback_service(ServeDir::new("./frontend/dist"))
                                         .route("/hello", get(|| async { "hi" }))
                                         .layer(cache_control_layer)
                                         .layer(CompressionLayer::new().quality(tower_http::CompressionLevel::Best))
                                         .layer(CacheLayer::with_lifespan(Duration::from_secs(CACHE_LIFETIME)));
    let router =
        axum::Router::new().merge(page_router)
                           .nest("/api", api_router)
                           .layer(TraceLayer::new_for_http())
                           .layer(DecompressionLayer::new())
                           .into_make_service();
    let config = RustlsConfig::from_pem_file("./secrets/daniel-gutierrez.com.pem", "./secrets/daniel-gutierrez.com.key")
                                .await
                                .expect("Failed to parse SSL Certificate");
    println!("Starting Server");
    let _server = axum_server::bind_rustls(addr,config).serve(router).await.unwrap();
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
                       next: Next)
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
