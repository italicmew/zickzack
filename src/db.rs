use rusqlite::{params, Connection};
use serde_derive::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub url: String,
    pub data: Option<Vec<u8>>,
}

impl Data {
    pub fn url_to_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let s = self.url.replace("/", "");
        hasher.update(s);
        format!("{:x}", hasher.finalize())
    }
}

pub struct Database {
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
                data BLOB
            )",
            [],
        )
    }

    pub fn save_data(&self, data: &Data) -> Result<(), rusqlite::Error> {
        println!("set: {}", data.url);
        let encoded_url = data.url_to_hash();
        self.conn.execute(
            "INSERT INTO data (id, url, data) 
            VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO UPDATE SET
                url = excluded.url,
                data = excluded.data;",
            params![encoded_url, data.url, data.data.as_deref()],
        )?;
        Ok(())
    }

    pub fn get_data(&self, url: String) -> Result<Option<Data>, rusqlite::Error> {
        let s = url.replace("/", "");
        let mut hasher = Sha256::new();
        hasher.update(&s);
        let id = format!("{:x}", hasher.finalize());
        let mut stmt = self
            .conn
            .prepare("SELECT id, url, data FROM data WHERE id = ?1")?;

        let mut data_iter = stmt.query_map(params![id], |row| {
            Ok(Data {
                url: row.get(1)?,
                data: row.get(2)?,
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
    fn test_database() {
        let db = Database::new(":memory:").unwrap();
        db.create_db().unwrap();

        let example_data = Data {
            url: "http://example.com".to_string(),
            data: Some(vec![1, 2, 3, 4, 5]),
        };

        db.save_data(&example_data).unwrap();

        let result = db
            .get_data("http://example.com".to_string())
            .expect("Failed to retrieve data from database");
        assert_eq!(
            result.map(|r| r.data),
            Some(example_data.data),
            "Retrieved data does not match saved data"
        );
    }

    #[test]
    fn hash_url_test() {
        let d = Data {
            url: "http://example.com".to_string(),
            data: Some(vec![1, 2, 3, 4, 5]),
        };
        assert_eq!(
            "2e897c7322de8382a9364ed19e301bae07a248fd251964c1c383d9b2183016c3",
            d.url_to_hash()
        );
    }
}
