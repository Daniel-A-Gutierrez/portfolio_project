#![feature(test)]
use std::fmt::Display;

use anyhow::{ensure, Result};
use rusqlite::{self, Connection, OptionalExtension, Row};

pub fn init_db() -> Result<Connection>
{
    let conn = Connection::open_in_memory()?;
    //define schema
    conn.execute("CREATE TABLE IF NOT EXISTS storage (id INTEGER PRIMARY KEY AUTOINCREMENT, 
                  unixtime INTEGER, key TEXT UNIQUE, value TEXT);",
                 ())?;
    return Ok(conn);
}

#[derive(Debug)]
pub enum SetError
{
    Oversized,
    KeyAlreadyPresent,
    DBError(rusqlite::Error),
}

impl Display for SetError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::Oversized => write!(f, "Error: Oversized Key Or Value"),
            Self::KeyAlreadyPresent => write!(f, "Error: Key Already Present"),
            Self::DBError(e) =>
            {
                println!("{:?}", e);
                write!(f, "Internal Server Error")
            }
        }
    }
}

pub fn db_set(conn: &Connection, key: &str, val: &str) -> Result<(), SetError>
{
    if key.len() > 10_000 || val.len() > 10_000
    {
        return Err(SetError::Oversized);
    };

    conn.execute("INSERT INTO storage (unixtime, key, value) VALUES (unixepoch(),$1,$2);",
                 (key, val))
        .map_err(|e| match e.sqlite_error_code()
        {
            Some(rusqlite::ErrorCode::ConstraintViolation) => SetError::KeyAlreadyPresent,
            _ => SetError::DBError(e),
        })?;
    return Ok(());
}

#[derive(Debug)]
pub struct KVRow
{
    pub id:       i64,
    pub unixtime: i64,
    pub key:      String,
    pub value:    String,
}

impl TryFrom<&Row<'_>> for KVRow
{
    type Error = anyhow::Error;
    fn try_from(row: &Row) -> std::result::Result<Self, Self::Error>
    {
        let id = row.get(0)?;
        let unixtime = row.get(1)?;
        let key = row.get(2)?;
        let value = row.get(3)?;
        return Ok(KVRow { id,
                          unixtime,
                          key,
                          value });
    }
}

pub fn db_get(conn: &Connection, key: &str) -> Result<Option<KVRow>>
{
    // if the key is too big it cant be in the database
    if key.len() > 10_000
    {
        return Ok(None);
    }
    let value = conn.query_one("SELECT * FROM storage WHERE key = $1 LIMIT 1;", (key,), |row| {
                        Ok(KVRow::try_from(row).expect("Row did not match table schema"))
                    })
                    .optional()?;
    return Ok(value);
}

#[cfg(test)]
mod test
{
    use super::*;
    extern crate test;
    use test::Bencher;
    #[test]
    fn initialization_t()
    {
        let cnxn = init_db().unwrap();
        cnxn.close().unwrap();
        return;
    }

    #[test]
    fn set_get_t()
    {
        let cnxn = init_db().unwrap();
        //just add "#[derive(Debug)]" above SetError
        db_set(&cnxn, "hi", "bye").unwrap();
        //forgot to take connection by reference! Should be db_get(conn: &Connection,
        let value = db_get(&cnxn, "hi").unwrap().unwrap().value;
        assert_eq!("bye", value);
    }

    #[bench]
    fn bencher(b: &mut Bencher)
    {
        let cnxn = init_db().unwrap();
        let mut start = 0;
        b.iter(|| {
             for keyi in start + 0..start + 1000
             {
                 let keys = keyi.to_string();
                 db_set(&cnxn, &keys, &keys).unwrap();
             }
             for keyi in start + 0..start + 1000
             {
                 let keys = keyi.to_string();
                 db_get(&cnxn, &keys).unwrap();
             }
             start += 1000;
         });
    }
}