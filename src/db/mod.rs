use rusqlite::Connection as SqliteConnection;
use std::sync::Mutex;

pub struct Db {
    conn: Mutex<SqliteConnection>,
}

impl Db {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = SqliteConnection::open(path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<(), rusqlite::Error> {
        let db_conn = self.conn.lock().unwrap();

        db_conn.execute(
            "CREATE TABLE IF NOT EXISTS connections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                host TEXT NOT NULL,
                port INTEGER NOT NULL DEFAULT 5432,
                user TEXT NOT NULL,
                password TEXT NOT NULL,
                db_name TEXT NOT NULL,
                schedule TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT ''
            )",
            [],
        )?;

        db_conn.execute(
            "CREATE TABLE IF NOT EXISTS logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                connection_id INTEGER,
                connection_name TEXT NOT NULL,
                status TEXT NOT NULL,
                message TEXT,
                file_path TEXT,
                created_at TEXT NOT NULL DEFAULT ''
            )",
            [],
        )?;

        Ok(())
    }

    pub fn get_all_connections(&self) -> Result<Vec<super::models::Connection>, rusqlite::Error> {
        let db_conn = self.conn.lock().unwrap();
        let mut stmt = db_conn.prepare(
            "SELECT id, name, host, port, user, password, db_name, schedule, enabled, created_at 
             FROM connections ORDER BY name",
        )?;

        let connections = stmt
            .query_map([], |row| {
                Ok(super::models::Connection {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    host: row.get(2)?,
                    port: row.get(3)?,
                    user: row.get(4)?,
                    password: row.get(5)?,
                    db_name: row.get(6)?,
                    schedule: row.get(7)?,
                    enabled: row.get::<_, i32>(8)? == 1,
                    created_at: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(connections)
    }

    pub fn get_enabled_connections(
        &self,
    ) -> Result<Vec<super::models::Connection>, rusqlite::Error> {
        let db_conn = self.conn.lock().unwrap();
        let mut stmt = db_conn.prepare(
            "SELECT id, name, host, port, user, password, db_name, schedule, enabled, created_at 
             FROM connections WHERE enabled = 1 ORDER BY name",
        )?;

        let connections = stmt
            .query_map([], |row| {
                Ok(super::models::Connection {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    host: row.get(2)?,
                    port: row.get(3)?,
                    user: row.get(4)?,
                    password: row.get(5)?,
                    db_name: row.get(6)?,
                    schedule: row.get(7)?,
                    enabled: row.get::<_, i32>(8)? == 1,
                    created_at: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(connections)
    }

    pub fn insert_connection(&self, c: &super::models::Connection) -> Result<i64, rusqlite::Error> {
        let db_conn = self.conn.lock().unwrap();
        db_conn.execute(
            "INSERT INTO connections (name, host, port, user, password, db_name, schedule, enabled) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                c.name,
                c.host,
                c.port,
                c.user,
                c.password,
                c.db_name,
                c.schedule,
                if c.enabled { 1 } else { 0 }
            ],
        )?;
        Ok(db_conn.last_insert_rowid())
    }

    pub fn update_connection(&self, c: &super::models::Connection) -> Result<(), rusqlite::Error> {
        let db_conn = self.conn.lock().unwrap();
        db_conn.execute(
            "UPDATE connections SET name=?1, host=?2, port=?3, user=?4, password=?5, 
             db_name=?6, schedule=?7, enabled=?8 WHERE id=?9",
            rusqlite::params![
                c.name,
                c.host,
                c.port,
                c.user,
                c.password,
                c.db_name,
                c.schedule,
                if c.enabled { 1 } else { 0 },
                c.id
            ],
        )?;
        Ok(())
    }

    pub fn delete_connection(&self, id: i64) -> Result<(), rusqlite::Error> {
        let db_conn = self.conn.lock().unwrap();
        db_conn.execute("DELETE FROM connections WHERE id=?1", [id])?;
        Ok(())
    }

    pub fn toggle_connection(&self, id: i64) -> Result<(), rusqlite::Error> {
        let db_conn = self.conn.lock().unwrap();
        db_conn.execute(
            "UPDATE connections SET enabled = NOT enabled WHERE id=?1",
            [id],
        )?;
        Ok(())
    }

    pub fn insert_log(&self, log: &super::models::Log) -> Result<i64, rusqlite::Error> {
        let db_conn = self.conn.lock().unwrap();
        db_conn.execute(
            "INSERT INTO logs (connection_id, connection_name, status, message, file_path) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                log.connection_id,
                log.connection_name,
                log.status,
                log.message,
                log.file_path
            ],
        )?;
        Ok(db_conn.last_insert_rowid())
    }

    pub fn get_logs(&self, limit: usize) -> Result<Vec<super::models::Log>, rusqlite::Error> {
        let db_conn = self.conn.lock().unwrap();
        let mut stmt = db_conn.prepare(
            "SELECT id, connection_id, connection_name, status, message, file_path, created_at 
             FROM logs ORDER BY created_at DESC LIMIT ?1",
        )?;

        let logs = stmt
            .query_map([limit as i64], |row| {
                Ok(super::models::Log {
                    id: row.get(0)?,
                    connection_id: row.get(1)?,
                    connection_name: row.get(2)?,
                    status: row.get(3)?,
                    message: row.get(4)?,
                    file_path: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    pub fn clear_logs(&self) -> Result<(), rusqlite::Error> {
        let db_conn = self.conn.lock().unwrap();
        db_conn.execute("DELETE FROM logs", [])?;
        Ok(())
    }
}
