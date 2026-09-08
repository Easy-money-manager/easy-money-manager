use rusqlite::Connection;
use chrono::NaiveDate;
use crate::record::Record;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn new(path: &str) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;

        Ok(Self { connection } )
    }
    pub fn initialize(&self) -> rusqlite::Result<()> {
        self.connection.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS collections (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sheets (
                id INTEGER PRIMARY KEY,
                collection_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                fraction INTEGER NOT NULL,

                FOREIGN KEY (collection_id)
                    REFERENCES collections(id)
            );

            CREATE TABLE IF NOT EXISTS records (
                id INTEGER PRIMARY KEY,
                sheet_id INTEGER NOT NULL,
                description TEXT NOT NULL,
                date TEXT NOT NULL,
                value INTEGER NOT NULL,

                FOREIGN KEY (sheet_id)
                    REFERENCES sheets(id)
            );
            ",
        )?;
        Ok(())
    }

    pub fn create_collection(&self, name: &str) -> rusqlite::Result<i64> {
        self.connection.execute("INSERT INTO collections (name) VALUES (?1)", [name])?;
        Ok(self.connection.last_insert_rowid())
    }
    pub fn get_collection(&self, name: &str) -> rusqlite::Result<i64> {
        self.connection.query_row("SELECT id FROM collections WHERE name = ?1", [name], |row| row.get(0))
    }
    pub fn get_or_create_collection(&self, name: &str) -> rusqlite::Result<i64> {
        match self.get_collection(name) {
            Ok(id) => Ok(id),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                self.create_collection(name)
            }
            Err(error) => Err(error),
        }
    }

    pub fn create_sheet(&self, collection_id: i64, name: &str, fraction: i64) -> rusqlite::Result<i64> {
        self.connection.execute("INSERT INTO sheets (collection_id, name, fraction) VALUES (?1, ?2, ?3)", (collection_id, name, fraction))?;
        Ok(self.connection.last_insert_rowid())
    }
    pub fn get_sheet(&self, collection_id: i64, name: &str) -> rusqlite::Result<i64> {
        self.connection.query_row("SELECT id FROM sheets WHERE collection_id = ?1 AND name = ?2", (collection_id, name), |row| row.get(0))
    }
    pub fn get_or_create_sheet(&self, collection_id: i64, name: &str, fraction: i64) -> rusqlite::Result<i64> {
        match self.get_sheet(collection_id, name) {
            Ok(id) => Ok(id),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                self.create_sheet(collection_id, name, fraction)
            }
            Err(error) => Err(error),
        }
    }

    pub fn create_record(&self, sheet_id: i64, description: &str, date: NaiveDate, value: i64) -> rusqlite::Result<i64> {
        self.connection.execute("INSERT INTO records (sheet_id, description, date, value) VALUES (?1, ?2, ?3, ?4)", (sheet_id, description, date.to_string(), value))?;
        Ok(self.connection.last_insert_rowid())
    }
    #[allow(dead_code)]
    pub fn get_record(&self, sheet_id: i64, description: &str, date: NaiveDate, value: i64) -> rusqlite::Result<i64> {
        self.connection.query_row("SELECT id from records WHERE sheet_id = ?1 and description = ?2 AND date = ?3 AND value = ?4", (sheet_id, description, date.to_string(), value), |row| row.get(0))
    }
    pub fn get_records(&self, sheet_id: i64) -> rusqlite::Result<Vec<Record>> {
        let mut statement = self.connection.prepare(
            "SELECT id, description, date, value
            FROM records
            WHERE sheet_id = ?1"
            )?;
        let records = statement.query_map(
            [sheet_id],
            |row| {
                let date_string: String = row.get(2)?;

                let date = NaiveDate::parse_from_str(&date_string, "%Y-%m-%d").expect("Failed to parse date from database's string");

                Ok(Record {
                    id: row.get(0)?,
                    description: row.get(1)?,
                    date,
                    value: row.get(3)?,
                })
            },
        )?;

        let mut result: Vec<Record> = Vec::new();
        for record in records{
            result.push(record?);
        }

        Ok(result)
    }
    pub fn update_record(&self, id: i64, description: &str, date: NaiveDate, value: i64) -> rusqlite::Result<()> {
        self.connection.execute(
            "UPDATE records
            SET description = ?1,
            date = ?2,
            value = ?3
            WHERE id = ?4",
            (description, date.to_string(), value, id)
            )?;

        Ok(())
    }
    pub fn remove_record(&self, id: i64) -> rusqlite::Result<()> {
        self.connection.execute(
            "DELETE FROM records
            WHERE id = ?1",
            [id]
        )?;

        Ok(())
    }
}
