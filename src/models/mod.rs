use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    #[serde(default)]
    pub id: i64,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub db_name: String,
    pub schedule: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub created_at: String,
}

fn default_enabled() -> bool {
    true
}

impl Connection {
    #[allow(dead_code)]
    pub fn new(
        name: String,
        host: String,
        port: u16,
        user: String,
        password: String,
        db_name: String,
        schedule: String,
    ) -> Self {
        Self {
            id: 0,
            name,
            host,
            port,
            user,
            password,
            db_name,
            schedule,
            enabled: true,
            created_at: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub id: i64,
    pub connection_id: i64,
    pub connection_name: String,
    pub status: String,
    pub message: String,
    pub file_path: Option<String>,
    pub created_at: String,
}
