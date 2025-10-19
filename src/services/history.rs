use std::sync::Mutex;

use anyhow::Result;

use crate::database::{Database, Transcription};

pub struct HistoryService {
    database: Mutex<Database>,
}

impl HistoryService {
    pub fn new(database: Database) -> Self {
        Self {
            database: Mutex::new(database),
        }
    }

    pub fn insert_transcription(
        &self,
        text: &str,
        processed_text: Option<&str>,
        language: Option<&str>,
        duration_ms: Option<i64>,
    ) -> Result<i64> {
        self.database
            .lock()
            .expect("database poisoned")
            .insert_transcription(text, processed_text, language, duration_ms)
    }

    pub fn recent(&self, limit: usize) -> Result<Vec<Transcription>> {
        self.database
            .lock()
            .expect("database poisoned")
            .get_recent_transcriptions(limit)
    }

    pub fn search(&self, query: &str) -> Result<Vec<Transcription>> {
        self.database
            .lock()
            .expect("database poisoned")
            .search_transcriptions(query)
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        self.database
            .lock()
            .expect("database poisoned")
            .delete_transcription(id)
    }
}
