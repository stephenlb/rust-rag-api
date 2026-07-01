// TODO Deduplication  
// TODO CHUNKING!!!!!
// TODO Tokio RUSQLITE!!!!<__ 
// TODO add Turbovec here
// TODO ✅ return Result<()> insert function
use rusqlite::{self, params, Connection};
use tokio::sync::Mutex;
use anyhow::Result;

const SCHEMA: &str = "
    CREATE VIRTUAL TABLE IF NOT EXISTS documents USING fts5 (
        text,
    );
";
const INSERT: &str = "
    INSERT INTO documents (text)
    VALUES (?1);
";
const SELECT: &str = "
    SELECT text, bm25(documents) AS rank
    FROM documents
    WHERE text MATCH ?1
    LIMIT ?2;
";

#[derive(Debug)] 
pub struct Database {
    connection: Mutex<Connection>,
}

#[derive(Debug)] 
pub struct Document {
    text: String,
    rank: f64,
}

impl Database {
    pub fn new() -> Self {
        let db = Connection::open_in_memory().expect("database connection");
        let _ = db.execute(SCHEMA, ());

        Self {
            connection: db.into(),
        }
    }

    pub async fn insert(&self, text: &str) -> Result<usize> {
        let guard = self.connection.lock().await;
        let result = guard.execute(INSERT, (text,))?;

        Ok(result)
    }

    pub async fn search(&self, search: &str, limit: i32) -> Result<String> {
        let guard = self.connection.lock().await;
        let mut statment = guard.prepare(SELECT)?;
        let documents = statment.query_map(params![search, limit], |row| {
            let text: String = row.get(0)?;
            let rank: f64 = row.get(1)?;
            dbg!(rank);
            Ok(Document { text, rank })
        })?;

        let mut docs: Vec<String> = vec![];
        for doc in documents {
            docs.push(doc?.text);
            //println!("{:?}", doc.unwrap());
        }

        Ok(docs.join("\n"))
    }
}
