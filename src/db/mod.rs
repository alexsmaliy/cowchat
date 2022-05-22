// One file can contain multiple modules. Imports are local to each module.
// Both modules and their members are subject to visibility rules.
// For a consumer to refer to the member of a module, both the module and the
// relevant member must be visible.
pub(crate) mod utils {

    use r2d2_sqlite::rusqlite;

    pub(crate) fn init_db_schema(conn: &rusqlite::Connection) {
        // Multiline strings are supported.
        conn.execute_batch("
            BEGIN;
            CREATE TABLE IF NOT EXISTS cows (
                cow_name VARCHAR(50) PRIMARY KEY,
                cow_id INTEGER UNIQUE,
                cow_color VARCHAR(20) NOT NULL,
                cow_age INTEGER NOT NULL,
                cow_weight INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS chat_sessions (
                chat_session_id INTEGER PRIMARY KEY,
                cow_id INTEGER,
                duration INTEGER NOT NULL,
                FOREIGN KEY(cow_id) REFERENCES cows (cow_id)
            );
            COMMIT;
        ").unwrap(); // TODO: better error handling?
    }
}

pub(crate) mod queries {
    // Constants need explicit type annotation.
    pub(crate) const LIST_COWS_QUERY: &str = "SELECT * FROM cows;";
    pub(crate) const CHECK_FOR_COW_QUERY: &str = "SELECT 0 <> (SELECT COUNT(*) FROM cows WHERE cow_name = :cow_name);";
    pub(crate) const COUNT_COWS_QUERY: &str = "SELECT COUNT(*) FROM cows;";
    pub(crate) const DISTINCT_COW_NAMES_QUERY: &str = "SELECT DISTINCT cow_name FROM cows;";
    pub(crate) const MAX_COW_ID_QUERY: &str = "SELECT COALESCE(MAX(cow_id), 0) FROM cows;";
    pub(crate) const INSERT_COW_QUERY: &str = "INSERT INTO
        cows (cow_name, cow_id, cow_color, cow_age, cow_weight)
        VALUES (:cow_name, :cow_id, :cow_color, :cow_age, :cow_weight);";
    pub(crate) const INSERT_CHAT_SESSION: &str = "INSERT INTO
        chat_sessions (cow_id, duration)
        SELECT cow_id, :duration FROM
            (SELECT cow_id FROM cows WHERE cow_name LIKE :cow_name COLLATE NOCASE);";
}

pub(crate) mod types {

    use r2d2::{Pool, PooledConnection};
    use r2d2_sqlite::SqliteConnectionManager;

    // Type aliases for long types.
    pub(crate) type MyPool = Pool<SqliteConnectionManager>;
    pub(crate) type MyConn = PooledConnection<SqliteConnectionManager>;
}
