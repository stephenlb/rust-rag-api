// TODO Docker
// TODO ratatui - UI FOR TERMINAL!!!! @foodude42
// TODO ✅ add Turbovec here
// TODO Propmt Stemming - remove filler words from prompt
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

const NUMBER_OF_WORDS_PER_CHUNK: usize = 200;
const DOCUMENT_DEDUPLICATION: &str = "
    CREATE TABLE IF NOT EXISTS document_deduplication (
        hash INT
    );
";

const DOCUMENT: &str = "
    CREATE VIRTUAL TABLE IF NOT EXISTS documents
    USING fts5 (text);
";
const INSERT: &str = "
    INSERT INTO documents (text)
    VALUES (?1);
";
const INSERT_DUPE: &str = "
    INSERT INTO document_deduplication (hash)
    VALUES (?1);
";
macro_rules! SELECT { () => {"
    select rowid, text, bm25(documents) AS rank
    FROM documents
    WHERE text MATCH ?1
    OR rowid in ({})
    LIMIT ?2;
"}; }

pub struct Database {
    connection: Mutex<Connection>,
    vector_store: Mutex<TurboQuantIndex>,
    embedding: Mutex<TextEmbedding>,
    cleaner: Cleaner,
}

#[derive(Debug)] 
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
        let vector_store = TurboQuantIndex::new(384, 4).unwrap();
        let embedding = TextEmbedding::try_new(Default::default()).unwrap();
        let cleaner = Cleaner::new();

        let _ = db.execute(DOCUMENT, ());
        let _ = db.execute(DOCUMENT_DEDUPLICATION, ());

        Self {
            connection: db.into(),
            vector_store: vector_store.into(),
            embedding: embedding.into(),
            cleaner: cleaner,
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
            let cleaned: String = self.cleaner.clean(&chunck);
            self.insert(&cleaned).await?;
        }
        Ok(0)
    }

    pub async fn insert(&self, text: &str) -> Result<usize> {
        if self.check_duplicate(text).await.is_err() {
            println!("DUPLICATE!!!!!!!!!");
            return Ok(0);
        }

        let input: Vec<&str> = vec![text];
        let mut embedding_guard = self.embedding.lock().await;
        let mut vector_guard = self.vector_store.lock().await;
        let guard = self.connection.lock().await;
        let vectors = embedding_guard.embed(input, None).unwrap();

        // Save to vector store
        vector_guard.add(&vectors[0]);
        
        // Save to Database
        let result = guard.execute(INSERT, (text,))?;
        let text_hash: i64 = hash(text);
        let result = guard.execute(INSERT_DUPE, params![text_hash])?;

        Ok(result)
    }

    pub async fn search(&self, search: &str, limit: i32) -> Result<String> {
        // Vector Sreach
        let cleaned: String = self.cleaner.clean(&search);
        let input: Vec<&str> = vec![&cleaned];
        let mut embedding_guard = self.embedding.lock().await;
        let vector_guard = self.vector_store.lock().await;
        let vectors = embedding_guard.embed(input, None).unwrap();
        let results = vector_guard.search(&vectors[0], 10);

        println!("Scores: {:?}", results.scores);
        println!("Indices: {:?}", results.indices);
        let rowids = results.indices.join(",");

        let select = format!(SELECT!(), rowids);
        let guard = self.connection.lock().await;
        let mut statment = guard.prepare(&select)?;
        let documents = statment.query_map(params![cleaned, limit], |row| {
            let rowid: i64 = row.get(0)?;
            let text: String = row.get(1)?;
            let rank: f64 = row.get(2)?;
            let _ = dbg!(rank);
            Ok(Document { rowid, text, rank })
        })?;


        let mut docs: Vec<String> = vec![];
        for doc in documents {
            // TODO fetch doc
            let doc = doc?;
            dbg!(doc.rowid);
            docs.push(doc.text);
        }

        Ok(docs.join("\n"))
    }
}
