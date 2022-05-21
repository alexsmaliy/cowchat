use std::collections::HashSet;

use actix_web::web::{Data, Json};
use anyhow::anyhow;
use log;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rand::prelude::*;

// `crate` is the root of import paths for local modules.
// Relative imports with `../` are also possible.
use crate::api::types::{BeckonCowsRequest, BeckonCowsResponse, Cow};
use crate::api::utils::{COW_NAMES, make_cow};
use crate::db::queries::{COUNT_COWS_QUERY, DISTINCT_COW_NAMES_QUERY, MAX_COW_ID_QUERY};
use crate::errors::CowError;

// Type alias for annoying types.
type MyPool = Pool<SqliteConnectionManager>;
type MyConn = PooledConnection<SqliteConnectionManager>;

pub(crate) async fn count_cows_handler(db_pool: Data<MyPool>) -> Result<String, CowError> {
    let conn = db_pool.get().map_err(|e| CowError::from(anyhow!(e)))?;
    match count_cows(&conn) {
        Err(e) => {
            log::error!("OMIGOD {}", e);
            Err(CowError::from(e))
        },
        // The OK arm is a no-op type conversion. In full, it would be like this:
        //    Result::<_, anyhow::Error>::Ok(value) => Result::<_, CowError>::Ok(value)
        Ok(value) => {
            log::debug!("Told client there were {} cows.", value);
            Ok(format!("{}", value))
        },
    }
}

pub(crate) async fn beckon_cows_handler(db_pool: Data<MyPool>, req: Json<BeckonCowsRequest>) -> Result<BeckonCowsResponse, CowError> {
    let conn = db_pool.get().map_err(|e| CowError::from(anyhow!(e)))?;
    match beckon_cows(&conn, req) {
        Err(e) => {
            log::error!("OMIGOD {}", e);
            Err(CowError::from(e))
        },
        Ok(value) => {
            log::debug!("Generated new cows: {}", value);
            Ok(value)
        }
    }
}

fn count_cows(conn: &MyConn) -> anyhow::Result<u32> {
    // let conn = db_pool.get()?;
    let mut stmt = conn.prepare_cached(COUNT_COWS_QUERY)?;
    let mut rows = stmt.query([])?; // this query takes no params
    let row = rows.next()?.ok_or(anyhow!("COUNT returned no rows!"))?;
    let count: u32 = row.get(0)?; // type annotation is required for get() to infer its return type
    Ok(count)
}

fn beckon_cows(conn: &MyConn, req: Json<BeckonCowsRequest>) -> anyhow::Result<BeckonCowsResponse> {
    let mut random = rand::thread_rng();
    let desired_number = req.count;
    let max_cows = COW_NAMES.len() as u32;
    let current_cows = count_cows(&conn)?;
    let adjusted_number = desired_number.min(max_cows - current_cows);
    if adjusted_number == 0 { anyhow::bail!("Insufficient cows in meadow! Let some go!") }
    let mut stmt = conn.prepare_cached(DISTINCT_COW_NAMES_QUERY)?;
    let used_names: HashSet<String> = stmt.query_map([], |row| row.get(0))?
        .map(|x: Result<String, _>| x.unwrap())
        .collect();
    let selected_names = COW_NAMES.difference(&used_names)
        .into_iter()
        .choose_multiple(&mut random, adjusted_number as usize);
    let mut stmt = conn.prepare_cached(MAX_COW_ID_QUERY)?;
    let max_id: u32 = stmt.query([])?.next()?.ok_or(anyhow!("MAX(cow_id) returned no rows!"))?.get(0)?;
    let new_cows: Vec<Cow> = selected_names.iter().enumerate().map(|(index, name)| {
        let next_id = max_id + index as u32 + 1;
        make_cow(name, next_id)
    }).collect();

    Ok(BeckonCowsResponse { cows: new_cows })
}

// fn generate_cow