// Import necessary items from the rusqlite crate and the standard library
use rusqlite::{params, Batch, Connection, Error, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

// Define a public struct called DbHandler that wraps a SQLite connection
pub struct DbHandler {
    conn: Connection,
}

impl DbHandler {
    // Constructor method for DbHandler
    // Takes a path (which can be any type that can be converted to a Path)
    // Returns a Result containing either a new DbHandler or an error
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(DbHandler { conn })
    }

    // Method to create necessary tables in the database
    // Uses the execute method to run SQL statements
    pub fn create_tables(&self) -> Result<()> {
        let sql = r"
        CREATE TABLE IF NOT EXISTS shows (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            show_type TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS episodes (
            id INTEGER PRIMARY KEY,
            show_id INTEGER,
            name TEXT NOT NULL,
            season INTEGER,
            episode_number INTEGER,
            UNIQUE(show_id, season, episode_number),
            FOREIGN KEY(show_id) REFERENCES shows(id)
        );
        CREATE TABLE IF NOT EXISTS transcripts (
            id INTEGER PRIMARY KEY,
            episode_id INTEGER,
            line_id INTEGER,
            time_start TEXT,
            time_end TEXT,
            text TEXT NOT NULL,
            UNIQUE(episode_id, time_start, time_end),
            FOREIGN KEY(episode_id) REFERENCES episodes(id)
        );
    ";

        let mut batch = Batch::new(&self.conn, sql);
        while let Some(mut stmt) = batch.next()? {
            stmt.execute([])?;
        }
        Ok(())
    }

    // Method to insert a new show into the database
    // Returns the ID of the newly inserted row
    // params! is a macro that helps prevent SQL injection
    pub fn insert_show(&self, name: &str, show_type: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO shows (name, show_type) VALUES (?1, ?2)",
            params![name, show_type],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    // Method to insert a new episode into the database
    pub fn insert_episode(
        &self,
        show_id: i64,
        name: &str,
        season: i32,
        episode_number: i32,
    ) -> Result<i64> {
        self.conn.execute(
        "INSERT OR IGNORE INTO episodes (show_id, name, season, episode_number) VALUES (?1, ?2, ?3, ?4)",
        params![show_id, name, season, episode_number],
    )?;
        Ok(self.conn.last_insert_rowid())
    }

    // Method to insert a new transcript group into the database
    pub fn insert_transcript(
        &self,
        episode_id: i64,
        line_id: i32,
        time_start: &str,
        time_end: &str,
        text: &str,
    ) -> Result<i64> {
        self.conn.execute(
        "INSERT OR IGNORE INTO transcripts (episode_id, line_id, time_start, time_end, text) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![episode_id, line_id, time_start, time_end, text],
    )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn batch_insert_shows(&mut self, shows: &[(String, String)]) -> Result<()> {
        println!("Inserting shows...");
        let sql = "INSERT OR IGNORE INTO shows (name, show_type) VALUES (?, ?)";
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(sql)?;
            for (name, show_type) in shows {
                stmt.execute(params![name, show_type])?;
            }
        }
        tx.commit()
    }

    pub fn batch_insert_episodes(&mut self, episodes: &[(i64, String, i32, i32)]) -> Result<()> {
        println!("Inserting episodes...");
        let sql =
        "INSERT OR IGNORE INTO episodes (show_id, name, season, episode_number) VALUES (?, ?, ?, ?)";
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(sql)?;
            for (show_id, name, season, episode_number) in episodes {
                stmt.execute(params![show_id, name, season, episode_number])?;
            }
        }
        tx.commit()
    }

    pub fn batch_insert_transcripts(
        &mut self,
        transcripts: &[(i64, i32, String, String, String)],
        output_csv: bool,
    ) -> Result<()> {
        println!("Inserting transcripts...");
        let sql = "INSERT OR IGNORE INTO transcripts (episode_id, line_id, time_start, time_end, text) VALUES (?, ?, ?, ?, ?)";
        let tx = self.conn.transaction()?;

        let mut csv_writer = if output_csv {
            Some(BufWriter::new(
                File::create("transcripts.csv")
                    .map_err(|e| Error::InvalidParameterName(e.to_string()))?,
            ))
        } else {
            None
        };

        {
            let mut stmt = tx.prepare(sql)?;
            for (episode_id, line_id, time_start, time_end, text) in transcripts {
                match stmt.execute(params![episode_id, line_id, time_start, time_end, text]) {
                    Ok(rows_affected) if rows_affected > 0 => {
                        let id = tx.last_insert_rowid();
                        if let Some(writer) = csv_writer.as_mut() {
                            for line in text.split('\n') {
                                writeln!(writer, "{},{}", id, line)
                                    .map_err(|e| Error::InvalidParameterName(e.to_string()))?;
                            }
                        }
                    }
                    Ok(_) => {}              // Row already exists, skip CSV writing
                    Err(e) => return Err(e), // Propagate other errors
                }
            }
        }

        if let Some(mut writer) = csv_writer {
            writer
                .flush()
                .map_err(|e| Error::InvalidParameterName(e.to_string()))?;
        }

        tx.commit()
    }
}
