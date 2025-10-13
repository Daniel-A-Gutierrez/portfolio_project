use std::net::SocketAddr;
use axum::{self,routing::{get,post}};
use tower_http;
use tokio;
use tower_http::services::ServeDir;
use anyhow::{Result, ensure};
use rusqlite::{Connection, self, OptionalExtension};

#[cfg(debug_assertions)]
const BIND : [u8;4] = [127,0,0,1];
#[cfg(debug_assertions)]
const PORT : u16 = 3334;
// release build will be out-facing
#[cfg(not(debug_assertions))]
const BIND : [u8;4] = [0,0,0,0];
#[cfg(not(debug_assertions))]
const PORT : u16 = 80; // or 443 for https if we cert up


#[tokio::main]
async fn main() 
{
    let addr = SocketAddr::from((BIND, PORT)); //0.0.0.0 for outfacing
    let page_router = axum::Router::new()
        .route("/hello", get( || async {"hello"}) )
        .nest_service("/",ServeDir::new("frontend/dist"))
        .into_make_service();
    let _server = hyper::Server::bind(&addr).serve(page_router).await.unwrap();
}

async fn init_db() -> Result<Connection>
{
    let conn = Connection::open_in_memory()?;
    conn.execute("CREATE TABLE IF NOT EXISTS storage (id : INTEGER PRIMARY KEY AUTOINCREMENT, 
                    keys : UNIQUE TEXT, values : TEXT);
                  CREATE UNIQUE INDEX IF NOT EXISTS idx_storage_keys ON storage (keys); ", ())?;
    return Ok(conn);
}

async fn db_truncate(conn : &Connection, to : usize) -> Result<()>
{
    conn.execute("DELETE FROM storage WHERE id > $1;", (to,))?;
    return Ok(());
}

async fn db_delete(conn : &Connection, key : &str) -> Result<()>
{
    conn.execute("DELETE FROM storage WHERE key = $1;", (key,))?;
    return Ok(());
}

enum InsertError { Oversized, KeyAlreadyPresent, DBError(rusqlite::Error) }

async fn db_insert<T>(conn : &Connection, key : &str, val : &str) -> Result<() , InsertError>
{
    if key.len() > 10_000 || val.len() > 10_000 { return Err(InsertError::Oversized) };

    conn.execute("INSERT INTO storage ($1,$2);", (key, val)).map_err(
        |e| {
            match e.sqlite_error_code() {
                Some(rusqlite::ErrorCode::ConstraintViolation) => InsertError::KeyAlreadyPresent, 
                _ => InsertError::DBError(e),
            }
        })?;
    return Ok(());
}

async fn db_get(conn : &Connection, key : &str) -> Result<Option<(usize, String)>, rusqlite::Error>
{
    return conn.query_one("SELECT id,value FROM storage WHERE key = $1;" , (key,) , 
        |row| Ok(Some((row.get(0).expect("Storage table has incorrect columns"), 
                  row.get(2).expect("Storage table has incorrect columns")))));
            
}