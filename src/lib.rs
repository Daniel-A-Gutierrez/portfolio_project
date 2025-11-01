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

#[derive(Debug, PartialEq)]
pub enum DBError
{
    Oversized,
    KeyAlreadyPresent,
    DBError(rusqlite::Error),
    KeyDoesntExist
}

impl Display for DBError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::Oversized => write!(f, "Error: Oversized Key Or Value"),
            Self::KeyAlreadyPresent => write!(f, "Error: Key Already Present"),
            Self::KeyDoesntExist => write!(f, "Error: Key Does Not Exist"),
            Self::DBError(e) =>
            {
                println!("{:?}", e);
                write!(f, "Internal Server Error")
            }
        }
    }
}

pub fn db_set(conn: &Connection, key: &str, val: &str) -> Result<(), DBError>
{
    if key.len() > 10_000 || val.len() > 10_000
    {
        return Err(DBError::Oversized);
    };

    conn.execute("INSERT INTO storage (unixtime, key, value) VALUES (unixepoch(),$1,$2);",
                 (key, val))
        .map_err(|e| match e.sqlite_error_code()
        {
            Some(rusqlite::ErrorCode::ConstraintViolation) => DBError::KeyAlreadyPresent,
            _ => DBError::DBError(e),
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

pub fn db_get(conn: &Connection, key: &str) -> Result<Option<KVRow>, DBError>
{
    // if the key is too big it cant be in the database
    if key.len() > 10_000
    {
        return Ok(None);
    }
    let value = conn.query_one("SELECT * FROM storage WHERE key = $1 LIMIT 1;", (key,), |row| {
                        Ok(KVRow::try_from(row).expect("Row did not match table schema"))
                    })
                    .optional().map_err(|e| DBError::DBError(e))?;
    return Ok(value);
}

#[rustfmt::ignore]
pub fn db_append(conn: &Connection, key: &str, val: &str) -> Result<(), DBError>
{
    if val.len() > 10_000 { return Err(DBError::Oversized); }
    let mut b4 = db_get(conn,key)?.ok_or_else(|| DBError::KeyDoesntExist)?;
    b4.value += val;
    if b4.key.len() + b4.value.len() > 20_000 {return Err(DBError::Oversized);}
    conn.execute("UPDATE storage SET value = $1 WHERE key = $2;",(b4.value,b4.key)) //doesnt support limit!
        .map_err(|e| DBError::DBError(e))?;
    return Ok(());
}

pub fn db_delete(conn: &Connection, key: &str) -> Result<(), DBError>
{
    if key.len() > 10_000 {return Err(DBError::Oversized);}
    conn.query_one("SELECT * FROM storage WHERE key = $1 LIMIT 1;", (key,), |row| {
                    Ok(KVRow::try_from(row).expect("Row did not match table schema"))
                })
                .optional()
                .map_err(|e| DBError::DBError(e))?
                .ok_or_else(|| DBError::KeyDoesntExist)?;
    conn.execute("DELETE FROM storage WHERE key = $1;",(key,))
        .map_err(|e| DBError::DBError(e))?;
    return Ok(());
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

    #[test]
    fn update_t()
    {
        let cnxn = init_db().unwrap();
        db_set(&cnxn, "hi", "bye").unwrap();
        db_append(&cnxn, "hi", "bye").unwrap();
        let value = db_get(&cnxn, "hi").unwrap().unwrap().value;
        assert_eq!("byebye", value);
    }

    #[test]
    fn update_neg_t()
    {
        let cnxn = init_db().unwrap();
        db_set(&cnxn, "hi", "bye").unwrap();
        let e = db_append(&cnxn, "hi2", "bye");
        assert!(e.is_err_and(|e| e == DBError::KeyDoesntExist));
    }

    #[test]
    fn delete_t()
    {
        let cnxn = init_db().unwrap();
        db_set(&cnxn, "hi", "bye").unwrap();
        db_delete(&cnxn, "hi").unwrap();
        let value = db_get(&cnxn, "hi").unwrap();
        assert!(value.is_none());
    }

    #[test]
    fn delete_neg_t()
    {
        let cnxn = init_db().unwrap();
        db_set(&cnxn, "hi", "bye").unwrap();
        let e = db_delete(&cnxn, "hii");
        assert!(e.is_err_and(|e| e==DBError::KeyDoesntExist));
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