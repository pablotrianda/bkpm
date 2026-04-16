use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use chrono::{DateTime, Datelike, FixedOffset, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::db::Db;
use crate::models::{Connection, Log};

fn now_timestamp() -> String {
    let now: DateTime<FixedOffset> = chrono::Local::now().into();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
}

pub fn router(db: Arc<Db>) -> Router {
    let state = AppState { db };

    Router::new()
        .route("/api/connections", get(get_connections).post(create_connection))
        .route(
            "/api/connections/:id",
            put(update_connection).delete(delete_connection),
        )
        .route("/api/connections/:id/toggle", post(toggle_connection))
        .route("/api/connections/:id/backup", post(run_backup_now))
        .route("/api/backups", get(get_backups))
        .route("/api/logs", get(get_logs).delete(clear_logs))
        .with_state(state)
}

async fn get_connections(State(state): State<AppState>) -> Json<Vec<Connection>> {
    Json(state.db.get_all_connections().unwrap_or_default())
}

async fn create_connection(
    State(state): State<AppState>,
    Json(conn): Json<Connection>,
) -> (StatusCode, Json<i64>) {
    match state.db.insert_connection(&conn) {
        Ok(id) => (StatusCode::CREATED, Json(id)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(-1)),
    }
}

async fn update_connection(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(mut conn): Json<Connection>,
) -> StatusCode {
    conn.id = id;
    match state.db.update_connection(&conn) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn delete_connection(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> StatusCode {
    match state.db.delete_connection(id) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn toggle_connection(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> StatusCode {
    match state.db.toggle_connection(id) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[derive(Serialize)]
struct BackupResult {
    success: bool,
    message: String,
    file_path: Option<String>,
}

async fn run_backup_now(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Json<BackupResult> {
    let connections = match state.db.get_all_connections() {
        Ok(c) => c,
        Err(_) => return Json(BackupResult {
            success: false,
            message: "Failed to get connections".to_string(),
            file_path: None,
        }),
    };

    let conn = match connections.iter().find(|c| c.id == id) {
        Some(c) => c.clone(),
        None => return Json(BackupResult {
            success: false,
            message: "Connection not found".to_string(),
            file_path: None,
        }),
    };

    let backup_dir = std::env::var("BACKUP_DIR").unwrap_or_else(|_| "/backups".to_string());
    let filename = format!(
        "{}_{:02}{:02}_{:02}{:02}_manual.sql",
        conn.name,
        chrono::Local::now().day(),
        chrono::Local::now().month(),
        chrono::Local::now().hour(),
        chrono::Local::now().minute()
    );

    let month_name = match chrono::Local::now().month() {
        1 => "01_enero",
        2 => "02_febrero",
        3 => "03_marzo",
        4 => "04_abril",
        5 => "05_mayo",
        6 => "06_junio",
        7 => "07_julio",
        8 => "08_agosto",
        9 => "09_setiembre",
        10 => "10_octubre",
        11 => "11_noviembre",
        12 => "12_diciembre",
        _ => "00_unknown",
    };

    let filepath = format!("{}/{}/{}/{}", backup_dir, conn.name, month_name, filename);
    let conn_id = conn.id;
    let conn_name = conn.name.clone();
    let db = state.db.clone();

    tokio::spawn(async move {
        let path = std::path::Path::new(&filepath);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let output = std::process::Command::new("pg_dump")
            .env("PGPASSWORD", &conn.password)
            .args([
                "-U", &conn.user,
                "-h", &conn.host,
                "-p", &conn.port.to_string(),
                "-d", &conn.db_name,
                "-F", "p",
            ])
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    if let Ok(mut file) = std::fs::File::create(&filepath) {
                        use std::io::Write;
                        let _ = file.write_all(&result.stdout);
                    }

                    let log = Log {
                        id: 0,
                        connection_id: conn_id,
                        connection_name: conn_name.clone(),
                        status: "success".to_string(),
                        message: "Manual backup completed".to_string(),
                        file_path: Some(filepath.clone()),
                        created_at: now_timestamp(),
                    };
                    let _ = db.insert_log(&log);
                    println!("Manual backup completed: {}", filepath);
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    let log = Log {
                        id: 0,
                        connection_id: conn_id,
                        connection_name: conn_name,
                        status: "error".to_string(),
                        message: format!("Manual backup failed: {}", stderr),
                        file_path: None,
                        created_at: now_timestamp(),
                    };
                    let _ = db.insert_log(&log);
                    println!("Manual backup failed: {}", stderr);
                }
            }
            Err(e) => {
                let log = Log {
                    id: 0,
                    connection_id: conn_id,
                    connection_name: conn_name,
                    status: "error".to_string(),
                    message: format!("Failed to run pg_dump: {}", e),
                    file_path: None,
                    created_at: now_timestamp(),
                };
                let _ = db.insert_log(&log);
                println!("Failed to run pg_dump: {}", e);
            }
        }
    });

    Json(BackupResult {
        success: true,
        message: "Backup started".to_string(),
        file_path: None,
    })
}

#[derive(Serialize, Deserialize, Clone)]
struct BackupFile {
    path: String,
    name: String,
    size: u64,
    created: String,
}

#[derive(Serialize)]
struct PaginatedBackups {
    data: Vec<BackupFile>,
    page: usize,
    limit: usize,
    total: usize,
    total_pages: usize,
}

async fn get_backups(Query(params): Query<HashMap<String, String>>) -> Json<PaginatedBackups> {
    let backup_dir = std::env::var("BACKUP_DIR").unwrap_or_else(|_| "/backups".to_string());
    let mut backups = Vec::new();

    scan_directory_recursive(&std::path::Path::new(&backup_dir), &mut backups);

    backups.sort_by(|a, b| b.created.cmp(&a.created));

    let total = backups.len();
    let limit: usize = params.get("limit")
        .and_then(|v: &String| v.parse().ok())
        .unwrap_or(10)
        .max(1)
        .min(100);
    let page: usize = params.get("page")
        .and_then(|v: &String| v.parse().ok())
        .unwrap_or(1)
        .max(1);
    let total_pages = (total + limit - 1) / limit;
    let actual_page = page.min(total_pages.max(1));

    let start = (actual_page - 1) * limit;
    let end = (start + limit).min(total);
    let data = if start < total {
        backups[start..end].to_vec()
    } else {
        Vec::new()
    };

    Json(PaginatedBackups {
        data,
        page: actual_page,
        limit,
        total,
        total_pages,
    })
}

fn scan_directory_recursive(path: &std::path::Path, backups: &mut Vec<BackupFile>) {
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Ok(meta) = entry.metadata() {
                        let created = meta
                            .created()
                            .map(|t| {
                                chrono::DateTime::<chrono::Local>::from(t)
                                    .format("%Y-%m-%d %H:%M:%S")
                                    .to_string()
                            })
                            .unwrap_or_default();

                        backups.push(BackupFile {
                            path: entry.path().to_string_lossy().to_string(),
                            name: entry.file_name().to_string_lossy().to_string(),
                            size: meta.len(),
                            created,
                        });
                    }
                } else if file_type.is_dir() {
                    scan_directory_recursive(&entry.path(), backups);
                }
            }
        }
    }
}

async fn get_logs(State(state): State<AppState>) -> Json<Vec<Log>> {
    Json(state.db.get_logs(50).unwrap_or_default())
}

async fn clear_logs(State(state): State<AppState>) -> StatusCode {
    match state.db.clear_logs() {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}