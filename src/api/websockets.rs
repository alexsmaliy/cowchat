use std::{
    time::{Duration, Instant}, sync::Arc,
};

// Some libraries have a prelude, so that they can dump all the relevant types,
// traits, and type conversions into your scope at once. Libraries can extend
// existing types using their own traits, but you must bring a trait into
// scope for it to add the new methods.
use actix::prelude::*;
use actix_web_actors::ws::{
    Message, ProtocolError, WebsocketContext,
};

use r2d2_sqlite::rusqlite::named_params;

use crate::api::utils::make_cow_phrase;
use crate::db::{
    types::MyPool, queries::INSERT_CHAT_SESSION,
};

const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

pub struct CowChat {
    started: Instant,
    heartbeat: Instant,
    // We give this type a reference to the connection pool instead of just a
    // single connection, because otherwise it would hold the connection for the
    // potentially unbounded length of an entire chat session.
    // An `Arc` is an asynchonous reference-counted pointer to a value, making
    // the value shareable between threads.
    db_pool: Arc<MyPool>,
    cow: String,
}

impl CowChat {
    pub fn new(db_pool: Arc<MyPool>, cow: &str) -> Self {
        let now = Instant::now();
        // Instant is Copy, so we can pass it by value to multiple consumers with impunity.
        // Foo { bar: bar } can be abbreviated to Foo { bar }.
        Self { started: now, heartbeat: now, db_pool, cow: String::from(cow) }
    }

    // For sotring the timestamp of the most recent ping or pong.
    fn refresh_heartbeat(&mut self) {
        self.heartbeat = Instant::now();
    }

    // Write some info about the chat to the DB when a chat ends.
    fn record_session_in_db(&self) {
        let conn = self.db_pool.get().unwrap();
        let mut stmt = conn.prepare_cached(INSERT_CHAT_SESSION).unwrap();
        // Duration overrides minus, so Duration - Duration = Duration.
        let duration = (self.heartbeat - self.started).as_secs();
        log::debug!("Recording chat session with {} that lasted for {} seconds...", self.cow, duration);
        // An if-let statement can also do destructuring.
        if let Err(e) = stmt.execute(named_params! {":cow_name": &self.cow, ":duration": duration}) {
            log::error!("Failed to record chat session in DB: {}", e);
        }
    }

    // Gets called when a session starts. <Foo as Bar> is the syntax for casting
    // a type to a trait that it implements, to access trait-specific fields or
    // methods (in this case, the associated Context type).
    fn start_beating(&self, context: &mut <CowChat as Actor>::Context) {
        context.run_interval(HEARTBEAT_INTERVAL, |actor, context| {
            if Instant::now().duration_since(actor.heartbeat) > CLIENT_TIMEOUT {
                log::warn!("Websocket client missed heartbeat, disconnecting!");
                context.stop();
            } else {
                // We ping single zero byte as a keep-alive every INTERVAL seconds.
                context.ping(&[b'0']);
            }
        });
    }
}

impl Actor for CowChat {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        self.start_beating(context);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        self.record_session_in_db();
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for CowChat {
    fn handle(&mut self, item: Result<Message, ProtocolError>, context: &mut Self::Context) {
        log::debug!("WS msg from client: {:?}", item);
        match item {
            Ok(Message::Ping(msg)) => {
                self.refresh_heartbeat();
                context.pong(&msg);
            },
            Ok(Message::Pong(_)) => {
                self.refresh_heartbeat();
            },
            Ok(Message::Binary(_)) => {
                log::warn!("Received unsupported binary message!");
            },
            Ok(Message::Text(_)) => {
                context.text(make_cow_phrase(&self.cow));
            },
            Ok(Message::Close(reason)) => {
                context.close(reason);
                context.stop();
            },
            _ => context.stop(),
        }
    }
}
