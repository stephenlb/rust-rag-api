// TODO test data
// TODO add LLM for responding in natuarl language
// TODO ratatui - UI FOR TERMINAL!!!! @foodude42
// TODO Docker
// TODO ✅ add Turbovec here
// TODO ✅ Propmt Stemming - remove filler words from prompt
// TODO ✅ CHUNKING!!!!!
// TODO ✅ build our own hash function "string" -> "s" -> 121 >> 5441
// TODO ✅ Hashing function - prevent duplication
// TODO ✅ Deduplication  
// TODO ✅ return Result<()> insert function
// TODO Tokio RUSQLITE!!!!<__ 
use crate::hash::*;
use crate::clean::Cleaner;
use rusqlite::{self, params, Connection, OptionalExtension};
use tokio::sync::Mutex;
use anyhow::Result;

// Vector Strore and Embedding
use turbovec::TurboQuantIndex;
use fastembed::{TextEmbedding, TextInitOptions, EmbeddingModel};


// Vector Score Thresholder
const SCORE_THRESHOLD: f32 = 0.60;

const NUMBER_OF_WORDS_PER_CHUNK: usize = 200;
const DOCUMENT_DEDUPLICATION: &str = "
    CREATE TABLE IF NOT EXISTS document_deduplication (
        hash INT PRIMARY KEY
    );
";

const DOCUMENT: &str = "
    CREATE VIRTUAL TABLE IF NOT EXISTS documents
    USING fts5 (text);
";
const INSERT: &str = "
    INSERT INTO documents (rowid, text)
    VALUES (?1, ?2);
";
const INSERT_DUPE: &str = "
    INSERT INTO document_deduplication (hash)
    VALUES (?1);
";
macro_rules! SELECT { () => {"
    SELECT rowid, text, BM25(documents) AS rank
    FROM documents
    WHERE text MATCH ?1
    OR rowid in ({})
    ORDER BY rank ASC
    LIMIT ?2;
"}; }

pub struct Database {
    connection: Mutex<Connection>,
    vector_store: Mutex<TurboQuantIndex>,
    embedding: Mutex<TextEmbedding>,
    cleaner: Cleaner,
}

// Full shape of an FTS5 result row. `rank` drives the SQL `ORDER BY`; `rowid`
// and `rank` are carried for callers/debugging even though search() currently
// only emits `text`.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Document {
    rowid: i64,
    text: String,
    rank: f64,
}

trait JoinVeci64 {
    fn join(&self, delimiter: &str) -> String;
}

impl JoinVeci64 for Vec<i64> {
    fn join(&self, delimiter: &str) -> String {
        let strings: Vec<String> = self
            .iter()
            .map(|n| n.to_string())
            .collect();

        strings.join(delimiter)
    }
}

impl Database {
    pub fn new() -> Self {
        let db = Connection::open_in_memory().expect("database connection");
        let vector_store = TurboQuantIndex::new(384, 4).expect("vector store init");
        let embedding = TextEmbedding::try_new(Default::default()).expect("embedding model init");
        let cleaner = Cleaner::new();

        let _ = db.execute(DOCUMENT, ());
        let _ = db.execute(DOCUMENT_DEDUPLICATION, ());

        Self {
            connection: db.into(),
            vector_store: vector_store.into(),
            embedding: embedding.into(),
            cleaner,
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
        let tail: &[String] = &words[number_of_chunks*NUMBER_OF_WORDS_PER_CHUNK..number_of_words];
        if !tail.is_empty() {
            chunks.push(tail.join(" "));
        }

        chunks
    }

    pub async fn add_document(&self, document: &str) -> Result<usize> {
        let chunks: Vec<String> = self.chunk(document);
        let mut inserted: usize = 0;
        for chunk in chunks {
            let cleaned: String = self.cleaner.clean(&chunk);
            self.insert(&cleaned).await?;
            inserted += 1;
        }
        Ok(inserted)
    }

    pub async fn insert(&self, text: &str) -> Result<i64> {
        if self.check_duplicate(text).await? {
            return Ok(0);
        }

        let input: Vec<&str> = vec![text];
        let mut embedding_guard = self.embedding.lock().await;
        let mut vector_guard = self.vector_store.lock().await;
        let guard = self.connection.lock().await;
        let vectors = embedding_guard.embed(input, None)?;

        // The vector lands in slot `vector_store.len()` (0-based); pin the FTS5
        // rowid to `slot + 1` so the two stays in lockstep by construction and
        // search() can map a slot back with a plain `+ 1` — no side table.
        let rowid: i64 = vector_guard.len() as i64 + 1;
        guard.execute(INSERT, params![rowid, text])?;
        let text_hash: i64 = hash(text);
        guard.execute(INSERT_DUPE, params![text_hash])?;

        vector_guard.add(&vectors[0]);

        Ok(rowid)
    }

    pub async fn search(&self, search: &str, limit: i32) -> Result<String> {
        // Vector search
        let cleaned: String = self.cleaner.clean(&search);
        let input: Vec<&str> = vec![&cleaned];
        let mut embedding_guard = self.embedding.lock().await;
        let vector_guard = self.vector_store.lock().await;
        let vectors = embedding_guard.embed(input, None)?;
        let results = vector_guard.search(&vectors[0], 10);

        // Keep hits above the score threshold. A vector in slot N was inserted
        // with FTS5 rowid N + 1 (see insert()), so the mapping is a plain `+ 1`.
        let rowids: Vec<i64> = results.indices
            .iter()
            .zip(results.scores.iter())
            .filter(|(_, score)| **score > SCORE_THRESHOLD)
            .map(|(slot, _)| slot + 1)
            .collect();

        let select = format!(SELECT!(), rowids.join(","));
        let guard = self.connection.lock().await;
        let mut statement = guard.prepare(&select)?;
        let documents = statement.query_map(params![cleaned, limit], |row| {
            let rowid: i64 = row.get(0)?;
            let text: String = row.get(1)?;
            let rank: f64 = row.get(2)?;
            Ok(Document { rowid, text, rank })
        })?;

        let mut docs: Vec<String> = vec![];
        for doc in documents {
            docs.push(doc?.text);
        }

        Ok(docs.join("\n"))
    }
}
