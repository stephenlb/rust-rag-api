use rusqlite::{self, Connection};
use tokio::sync::Mutex;
use anyhow::Result;

const SCHEMA: &str = "
    -- CREATE TABLE IF NOT EXISTS slice (
    --     id INT PRIMARY KEY,
    --     created DATETIME,
    --     segment TEXT
    -- );
    CREATE VIRTUAL TABLE IF NOT EXISTS slice USING fts5 (
        text,
    );
";
const INSERT: &str = "
    INSERT INTO slice (text)
    VALUES (?1)
";

#[derive(Debug)] 
pub struct Database {
    connection: Mutex<Connection>,
}

impl Database {
   pub fn new() -> Self {
        let db = Connection::open_in_memory().expect("database connection");
        let _ = db.execute(SCHEMA, ());

        Self {
            connection: db.into(),
        }
   }
   pub async fn insert(&self, text: &str) {
        let guard = self.connection.lock().await;
        let _ = guard.execute(INSERT, (text,));
   }
}
