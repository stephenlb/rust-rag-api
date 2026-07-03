// TODO Docker
// TODO add Turbovec here
// TODO ✅ CHUNKING!!!!!
// TODO ✅ build our own hash function "string" -> "s" -> 121 >> 5441
// TODO ✅ Hashing function - prevent duplication
// TODO ✅ Deduplication  
// TODO ✅ return Result<()> insert function
// TODO Tokio RUSQLITE!!!!<__ 
use crate::hash::*;
use rusqlite::{self, params, Connection, OptionalExtension};
use tokio::sync::Mutex;
use anyhow::Result;

const NUMBER_OF_WORDS_PER_CHUNK: usize = 200;
const DOCUMENT_DEDUPLICATION: &str = "
    CREATE TABLE IF NOT EXISTS document_deduplication (
        hash INT
    );
";
const DOCUMENT: &str = "
    CREATE VIRTUAL TABLE IF NOT EXISTS documents USING fts5 (
        text
    );
";
const INSERT: &str = "
    INSERT INTO documents (text)
    VALUES (?1);
";
const INSERT_DUPE: &str = "
    INSERT INTO document_deduplication (hash)
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
        let _ = db.execute(DOCUMENT, ());
        let _ = db.execute(DOCUMENT_DEDUPLICATION, ());

        Self {
            connection: db.into(),
        }
    }

    async fn check_duplicate(&self, text: &str) -> Result<bool> {
        let guard = self.connection.lock().await;
        let text_hash: i64 = hash(text);
	let duplicate: Option<String> = guard.query_row(
            "SELECT hash FROM document_deduplication WHERE hash = ?1 LIMIT 1;",
            params![text_hash],
            |row| row.get(0),
        ).optional()?;

        Ok(duplicate.is_some())
    }

    fn chunk(&self, text: &str) -> Vec<String> {
        let mut chunks: Vec<String> = vec![];
        let mut current_word: usize = 0;
        let words: Vec<String> = text
            .split_whitespace()
            .map(|w| w.to_string())
            .collect();
        let number_of_words: usize = words.len();
        let number_of_chunks: usize = number_of_words / NUMBER_OF_WORDS_PER_CHUNK;

        for chunk in 0..number_of_chunks {
            let sentence: String = words[
                chunk*NUMBER_OF_WORDS_PER_CHUNK..
                (chunk+1)*NUMBER_OF_WORDS_PER_CHUNK
            ].join(" ");
            chunks.push(sentence);
        }
        let sentence: String = words[number_of_chunks*NUMBER_OF_WORDS_PER_CHUNK..number_of_words].join(" ");
        chunks.push(sentence);

        chunks
    }

    pub async fn add_document(&self, document: &str) -> Result<usize> {
        let chunks: Vec<String> = self.chunk(document);
        for chunck in chunks {
            self.insert(&chunck).await?;
        }
        Ok(0)

    }

    pub async fn insert(&self, text: &str) -> Result<usize> {
        if self.check_duplicate(text).await.is_err() {
            println!("DUPLICATE!!!!!!!!!");
            return Ok(0);
        }
        
        let guard = self.connection.lock().await;
        let result = guard.execute(INSERT, (text,))?;
        let text_hash: i64 = hash(text);
        let result = guard.execute(INSERT_DUPE, params![text_hash])?;

        Ok(result)
    }

    pub async fn search(&self, search: &str, limit: i32) -> Result<String> {
        let guard = self.connection.lock().await;
        let mut statment = guard.prepare(SELECT)?;
        let documents = statment.query_map(params![search, limit], |row| {
            let text: String = row.get(0)?;
            let rank: f64 = row.get(1)?;
            let _ = dbg!(rank);
            Ok(Document { text, rank })
        })?;

        let mut docs: Vec<String> = vec![];
        for doc in documents {
            docs.push(doc?.text);
        }

        Ok(docs.join("\n"))
    }
}
