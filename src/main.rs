// Library imports. Imports can be glommed.
use actix_web::{App, HttpServer, middleware::{Logger, NormalizePath}, web::{Data, get, post, scope}};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

// My local imports, separated for clarity.
use api::handlers::{count_cows_handler, beckon_cows_handler};
use db::utils::init_db_schema;

// Declarations of modules that are direct descendants of this one.
// In Rust, a module declares its children. No multi-level declarations.
mod api;
mod db;
mod errors;

const NUM_WORKERS: u32 = 5;

// This annotation is required so that Actix can rewrite the async main() into
// what Rust actually ends up running, which is a sync main() that awaits this
// async function. Rust main() is normally not async.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_log();

    let manager = SqliteConnectionManager::file("cowchat.db");
    let pool = Pool::builder()
        .min_idle(Some(NUM_WORKERS)) // This arg can also be Option::None, hence Option::Some(N).
        .build(manager)
        .unwrap();
    init_db_schema(&pool.get().unwrap());

    // We create the DB connection pool once and issue references to it to each
    // copy of the multithreaded application. `Data` is the Actix thread-safe box
    // for sharing stuff between threads. Clones of `Data` are just clones of the
    // pointer, not the pool itself.
    let shared_pool = Data::new(pool);

    // This closure initializes each server thread with the application logic.
    // Each app thread is self-contained, so it "eats" all references it needs
    // from the parent scope instead of just referring to them. 
    let app_factory = move || {
        let logger = Logger::default();

        let cows_scope = scope("/cows").route("/count", get().to(count_cows_handler))
                                       .route("/beckon", post().to(beckon_cows_handler));

        App::new().app_data(shared_pool.clone()) // shared stuff
                  .wrap(logger) // logging middleware
                  .wrap(NormalizePath::trim()) // middleware to trim trailing slashes
                  .service(cows_scope) // routing
    };

    let host_port = ("localhost", 3000);
    
    HttpServer::new(app_factory)
        .workers(NUM_WORKERS as usize) // no automatic conversions between numeric types in Rust
        .bind(host_port)? // `?` unwraps fallible operation
        .run()
        .await // Awaiting the server (which is basically a Promise) is what makes it run.
}

fn init_log() {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
}