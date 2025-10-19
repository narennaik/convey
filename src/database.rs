use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transcription {
    pub id: i64,
    pub text: String,
    pub processed_text: Option<String>,
    pub language: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path).context("Failed to open database")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL,
                processed_text TEXT,
                language TEXT,
                duration_ms INTEGER,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn insert_transcription(
        &self,
        text: &str,
        processed_text: Option<&str>,
        language: Option<&str>,
        duration_ms: Option<i64>,
    ) -> Result<i64> {
        let created_at = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO transcriptions (text, processed_text, language, duration_ms, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![text, processed_text, language, duration_ms, created_at],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_recent_transcriptions(&self, limit: usize) -> Result<Vec<Transcription>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, text, processed_text, language, duration_ms, created_at
             FROM transcriptions
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;

        let transcriptions = stmt
            .query_map([limit], |row| {
                Ok(Transcription {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    processed_text: row.get(2)?,
                    language: row.get(3)?,
                    duration_ms: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transcriptions)
    }

    pub fn search_transcriptions(&self, query: &str) -> Result<Vec<Transcription>> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, text, processed_text, language, duration_ms, created_at
             FROM transcriptions
             WHERE text LIKE ?1 OR processed_text LIKE ?1
             ORDER BY created_at DESC
             LIMIT 100",
        )?;

        let transcriptions = stmt
            .query_map([&search_pattern], |row| {
                Ok(Transcription {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    processed_text: row.get(2)?,
                    language: row.get(3)?,
                    duration_ms: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transcriptions)
    }

    pub fn delete_transcription(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM transcriptions WHERE id = ?1", params![id])?;
        Ok(())
    }
}
