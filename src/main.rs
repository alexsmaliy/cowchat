// Library imports. Imports can be glommed.
use actix_web::{
    App, HttpServer,
    middleware::{Logger, NormalizePath},
    web::{Data, get, post, scope},
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

// My local imports, separated for clarity.
use api::handlers::{
    count_cows_handler, beckon_cows_handler, list_cows_handler,
    websocket_cowchat_handler,
};
use db::utils::init_db_schema;

// Declarations of modules that are direct descendants of this one.
// In Rust, a module declares its children. No multi-level declarations.
mod api;
mod db;
mod errors;

// Const values must be evaluable at compile-time, so they are quite limited.
const NUM_WORKERS: u32 = 5;

// This annotation is required so that Actix can rewrite the async main() into
// what Rust actually ends up running. Rust main() is normally not async.
#[actix_web::main]
async fn main() -> std::io::Result<()> { // Functions are required to declare input/output types.
    init_log();

    // Type::function is static functions, instance.function is instance methods.
    let manager = SqliteConnectionManager::file("cowchat.db");
    let pool = Pool::builder()
        .min_idle(Some(NUM_WORKERS)) // This arg can also be Option::None, hence Option::Some(N).
        .build(manager)
        .unwrap();
    // unwrap() works on Result and Option types and basically means
    // "I don't want to do error handling." If the unwrapped value is Err, the
    // program just crashes.
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

        // A "scope" in this case s just a group of routes.
        let cows_scope = scope("/cows").route("/count", get().to(count_cows_handler))
                                       .route("/beckon", post().to(beckon_cows_handler))
                                       .route("/list", get().to(list_cows_handler))
                                       .route("/chat/{cow_name}", get().to(websocket_cowchat_handler));

        App::new().app_data(shared_pool.clone()) // shared stuff
                  .wrap(logger) // logging middleware
                  .wrap(NormalizePath::trim()) // middleware to trim trailing slashes from paths
                  .service(cows_scope) // routing
    };

    // A tuple.
    let host_port = ("localhost", 3000);
    
    HttpServer::new(app_factory)
        // no automatic conversions between numeric types in Rust
        .workers(NUM_WORKERS as usize)
        // `?` unwraps Result/Option values and returns the Err value from the function immediately
        .bind(host_port)?
        .run()
        // Awaiting the server (which is basically a Promise) is what makes it run.
        .await
}

fn init_log() {
    // log levels include trace/debug/info/warn/error/off
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
}
