use rusqlite::Connection;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn new(path: &str) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;

        connection.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS collections (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sheets (
                id INTEGER PRIMARY KEY,
                collection_id INTEGER NOT NULL,
                name TEXT NOT NULL,

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
        Ok(Self { connection })
    }
    pub fn initialize(&self) -> rusqlite::Result<()> {
        let main_id: i64 = self.get_or_create_collection("Main")?;
        self.get_or_create_sheet(main_id, "Incomes")?;
        self.get_or_create_sheet(main_id, "Essentials")?;
        self.get_or_create_sheet(main_id, "Stability")?;
        self.get_or_create_sheet(main_id, "Growth")?;
        self.get_or_create_sheet(main_id, "Other")?;
        let plan_id: i64 = self.get_or_add_collection("Planning")?;
        self.get_or_create_sheet(plan_id, "Incomes")?;
        self.get_or_create_sheet(plan_id, "Expenses")?;
        println!("{:?}", self.get_collection("Main").unwrap());
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
    pub fn create_sheet(&self, collection_id: i64, name: &str) -> rusqlite::Result<i64> {
        self.connection.execute("INSERT INTO sheets (collection_id, name) VALUES (?1, ?2)", (collection_id, name))?;
        Ok(self.connection.last_insert_rowid())
    }
    pub fn get_sheet(&self, collection_id: i64, name: &str) -> rusqlite::Result<i64> {
        self.connection.query_row("SELECT id FROM sheets WHERE collection_id = ?1 AND name = ?2", (collection_id, name), |row| row.get(0))
    }
    pub fn get_or_create_sheet(&self, collection_id: i64, name: &str) -> rusqlite::Result<i64> {
        match self.get_sheet(collection_id, name) {
            Ok(id) => Ok(id),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                self.create_sheet(collection_id, name)
            }
            Err(error) => Err(error),
        }
    }
//    pub fn create_record(&self) {
//    }
}
