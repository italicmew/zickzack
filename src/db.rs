use rusqlite::{Connection, params};
use serde_derive::{Serialize, Deserialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    url: String,
    data: Option<Vec<u8>>,
    nonce: Option<Vec<u8>>,
}

struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;
        Ok(Database { conn })
    }

    pub fn create_db(&self) -> Result<usize, rusqlite::Error> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS data (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                data BLOB,
                nonce BLOB
            )",
            [],
        )
    }

    pub fn save_data(&self, data: &Data) -> Result<(), rusqlite::Error> {
        let mut hasher = Sha256::new();
        hasher.update(&data.url);
        let id = format!("{:x}", hasher.finalize());

        self.conn.execute(
            "INSERT INTO data (id, url, data, nonce) VALUES (?1, ?2, ?3, ?4)",
            params![id, data.url, data.data.as_deref(), data.nonce.as_deref()],
        )?;
        Ok(())
    }

    pub fn get_data(&self, url: String) -> Result<Option<Data>, rusqlite::Error> {

        let mut hasher = Sha256::new();
        hasher.update(&url);
        let id = format!("{:x}", hasher.finalize());
        let mut stmt = self.conn.prepare("SELECT id, url, data, nonce FROM data WHERE id = ?1")?;

        let mut data_iter = stmt.query_map(params![id], |row| {
            Ok(Data {
                url: row.get(1)?,
                data: row.get(2)?,
                nonce: row.get(3)?,
            })
        })?;

        if let Some(data_row) = data_iter.next() {
            data_row.map(Some)
        } else {
            Ok(None)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let db = Database::new(":memory:").unwrap();
        db.create_db().unwrap();

        let example_data = Data {
            url: "http://example.com".to_string(),
            data: Some(vec![1, 2, 3, 4, 5]),
            nonce: Some(vec![5, 4, 3, 2, 1]),
        };

        db.save_data(&example_data).unwrap();

        let result = db.get_data("http://example.com".to_string()).expect("Failed to retrieve data from database");
        assert_eq!(result.map(|r| r.data), Some(example_data.data), "Retrieved data does not match saved data");
    }
}