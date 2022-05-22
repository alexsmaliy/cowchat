use std::collections::HashSet;

use actix_web::{error, HttpRequest, HttpResponse};
use actix_web::web::{Data, Json, Path, Payload};
use actix_web_actors::ws;
use anyhow::anyhow;

use r2d2_sqlite::{rusqlite, rusqlite::named_params};
use rand::prelude::*;

// `crate` is the root of import paths for local modules.
// Relative imports with `../` are also possible.
use crate::api::types::{BeckonCowsRequest, CowListResponse, Cow, CowColor};
use crate::api::utils::{COW_NAMES, make_cow};
use crate::api::websockets::CowChat;
use crate::db::queries::{CHECK_FOR_COW_QUERY, COUNT_COWS_QUERY, DISTINCT_COW_NAMES_QUERY, INSERT_COW_QUERY, LIST_COWS_QUERY, MAX_COW_ID_QUERY};
use crate::db::types::{MyConn, MyPool};
use crate::errors::CowError;    

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

pub(crate) async fn beckon_cows_handler(db_pool: Data<MyPool>, req: Json<BeckonCowsRequest>) -> Result<CowListResponse, CowError> {
    let conn = db_pool.get().map_err(|e| CowError::from(anyhow!(e)))?;
    match beckon_cows(&conn, req) {
        Err(e) => {
            log::error!("{}", e);
            Err(CowError::from(e))
        },
        Ok(cows) => {
            let s = cows.iter().map(|c| format!("{}", c)).collect::<Vec<String>>().join(", ");
            log::debug!("Generated new cows: {}", s);
            Ok(CowListResponse { cows })
        }
    }
}

pub(crate) async fn list_cows_handler(db_pool: Data<MyPool>) -> Result<CowListResponse, CowError> {
    let conn = db_pool.get().map_err(|e| CowError::from(anyhow!(e)))?;
    match list_cows(&conn) {
        Err(e) => {
            log::error!("{}", e);
            Err(CowError::from(e))
        },
        Ok(cows) => {
            log::debug!("Reporting on {} existing cows to client.", cows.len());
            Ok(CowListResponse { cows })
        }
    }
}

pub(crate) async fn websocket_cowchat_handler(db_pool: Data<MyPool>, path: Path<String>, req: HttpRequest, stream: Payload) -> Result<HttpResponse, error::Error> {
    let pool_ref = (*db_pool).clone();
    let cow_name = capitalized(&path.into_inner());
    let conn = db_pool.get().map_err(error::ErrorInternalServerError)?;
    if check_for_cow(&conn, &cow_name).map_err(error::ErrorInternalServerError)? {
        ws::start(CowChat::new(pool_ref, cow_name), &req, stream)
    } else {
        Err(error::ErrorBadRequest(anyhow!("No such cow currently present to chat with: {}", cow_name)))
    }
}

fn capitalized(s: &str) -> String {
    let mut cs = s.chars();
    cs.next().unwrap().to_uppercase().to_string() + cs.as_str()
}

fn check_for_cow(conn: &MyConn, cow_name: &str) -> Result<bool, CowError> {
    let stmt = conn.prepare_cached(CHECK_FOR_COW_QUERY);
    let row: Result<u32, rusqlite::Error> = stmt.and_then(|mut stmt| {
        stmt.query_row(&[(":cow_name", &cow_name)], |row| row.get(0))
    });
    row.map(|val| val == 1).map_err(|e| CowError::from(anyhow!(e)))
}

fn count_cows(conn: &MyConn) -> anyhow::Result<u32> {
    let mut stmt = conn.prepare_cached(COUNT_COWS_QUERY)?;
    let mut rows = stmt.query([])?; // this query takes no params
    let row = rows.next()?.ok_or_else(|| anyhow!("COUNT returned no rows!"))?;
    let count: u32 = row.get(0)?; // type annotation is required for get() to infer its return type
    Ok(count)
}

fn list_current_cow_names(conn: &MyConn) -> anyhow::Result<HashSet<String>> {
    let mut stmt = conn.prepare_cached(DISTINCT_COW_NAMES_QUERY)?;
    let used_names: HashSet<String> = stmt.query_map([], |row| row.get(0))?
        .map(|x: Result<String, _>| x.unwrap())
        .collect();
    Ok(used_names)
}

fn list_cows(conn: &MyConn) -> anyhow::Result<Vec<Cow>> {
    let mut stmt = conn.prepare_cached(LIST_COWS_QUERY)?;
    let cows: Vec<Cow> = stmt.query_map([], |row| {
                let name: String = row.get_unwrap(0);
                let id: u32 = row.get_unwrap(1);
                let color: CowColor = row.get_unwrap(2);
                let age: u32 = row.get_unwrap(3);
                let weight: u32 = row.get_unwrap(4);
                Ok(Cow::new(name, id, color, age, weight))
            })?
        .map(|x: Result<Cow, _>| x.unwrap())
        .collect();
    Ok(cows)
}

fn get_current_max_id(conn: &MyConn) -> anyhow::Result<u32> {
    let mut stmt = conn.prepare_cached(MAX_COW_ID_QUERY)?;
    let max_id: u32 = stmt.query([])?
                          .next()?
                          .ok_or_else(|| anyhow!("MAX(cow_id) returned no rows!"))?
                          .get(0)?;
    Ok(max_id)
}

fn write_cows(conn: &MyConn, cows: &Vec<Cow>) -> anyhow::Result<()> {
    let mut stmt = conn.prepare_cached(INSERT_COW_QUERY)?;
    for cow in cows {
        let Cow { id, name, color, age, weight} = cow;
        stmt.execute(named_params! {
            ":cow_name": name,
            ":cow_id": id,
            ":cow_color": color,
            ":cow_age": age,
            ":cow_weight": weight,
        })?;
    }
    Ok(())
}

fn beckon_cows(conn: &MyConn, req: Json<BeckonCowsRequest>) -> anyhow::Result<Vec<Cow>> {
    let mut random = rand::thread_rng();
    let desired_number = req.count;
    let max_cows = COW_NAMES.len() as u32;
    let current_cows = count_cows(conn)?;
    let adjusted_number = desired_number.min(max_cows - current_cows);
    if adjusted_number == 0 { anyhow::bail!("Insufficient cows in meadow! Let some go!") }
    let used_names = list_current_cow_names(conn)?;
    let chosen_available_names = COW_NAMES.difference(&used_names)
        .into_iter()
        .choose_multiple(&mut random, adjusted_number as usize);
    let max_id = get_current_max_id(conn)?;
    let new_cows: Vec<Cow> = chosen_available_names.iter().enumerate().map(|(index, name)| {
        let next_id = max_id + index as u32 + 1;
        make_cow(name, next_id)
    }).collect();
    let write_outcome = write_cows(conn, &new_cows);
    write_outcome.or_else(|e| anyhow::bail!("Could not write cows to database: {}", e))?;
    Ok(new_cows)
}
