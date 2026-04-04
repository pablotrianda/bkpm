use chrono::{DateTime, Datelike, FixedOffset, Local, Timelike};
use std::io::Write;
use std::process::Command;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::db::Db;
use crate::models::{Connection, Log};

pub struct Scheduler {
    stop_tx: Option<mpsc::Sender<()>>,
}

fn now_timestamp() -> String {
    let now: DateTime<FixedOffset> = Local::now().into();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

impl Scheduler {
    pub fn new() -> Self {
        Self { stop_tx: None }
    }

    pub fn start(&mut self, db: Arc<Db>, backup_dir: String) {
        let (stop_tx, stop_rx) = mpsc::channel();
        self.stop_tx = Some(stop_tx);

        thread::spawn(move || {
            let mut last_run: std::collections::HashMap<i64, i64> =
                std::collections::HashMap::new();

            loop {
                if stop_rx.try_recv().is_ok() {
                    break;
                }

                let connections = match db.get_enabled_connections() {
                    Ok(c) => c,
                    Err(_) => {
                        thread::sleep(Duration::from_secs(60));
                        continue;
                    }
                };

                if connections.is_empty() {
                    thread::sleep(Duration::from_secs(60));
                    continue;
                }

                let now = Local::now();
                let current_minute = now.minute() as i64 + (now.hour() as i64 * 60);

                for conn in &connections {
                    let last_minute = last_run.get(&conn.id).copied();
                    let should_run = Self::should_run_now(&conn.schedule);

                    if Some(current_minute) != last_minute && should_run {
                        println!("Running backup for {}", conn.name);
                        let filename = Self::generate_filename(&conn.name);
                        let filepath = Self::generate_filepath(&backup_dir, &conn.name, &filename);

                        let log_id =
                            Self::create_log(&db, conn, "running", "Starting backup...", None);

                        match Self::run_backup(&conn, &filepath) {
                            Ok(_) => {
                                println!("Backup completed: {}", filepath);
                                last_run.insert(conn.id, current_minute);
                                Self::update_log(
                                    &db,
                                    log_id,
                                    "success",
                                    "Backup completed",
                                    Some(&filepath),
                                );
                            }
                            Err(e) => {
                                println!("Backup failed for {}: {}", conn.name, e);
                                last_run.insert(conn.id, current_minute);
                                Self::update_log(&db, log_id, "error", &e, None);
                            }
                        }
                    }
                }

                thread::sleep(Duration::from_secs(60));
            }
        });
    }

    fn create_log(
        db: &Arc<Db>,
        conn: &Connection,
        status: &str,
        message: &str,
        file_path: Option<&str>,
    ) -> i64 {
        let log = Log {
            id: 0,
            connection_id: conn.id,
            connection_name: conn.name.clone(),
            status: status.to_string(),
            message: message.to_string(),
            file_path: file_path.map(|s| s.to_string()),
            created_at: now_timestamp(),
        };
        db.insert_log(&log).unwrap_or(0)
    }

    fn update_log(db: &Arc<Db>, id: i64, status: &str, message: &str, file_path: Option<&str>) {
        let log = Log {
            id,
            connection_id: 0,
            connection_name: String::new(),
            status: status.to_string(),
            message: message.to_string(),
            file_path: file_path.map(|s| s.to_string()),
            created_at: now_timestamp(),
        };
        let _ = db.insert_log(&log);
    }

    fn should_run_now(schedule: &str) -> bool {
        let parts: Vec<&str> = schedule.split_whitespace().collect();
        if parts.len() != 5 {
            return false;
        }

        let now = Local::now();
        let minute = now.minute();
        let hour = now.hour();
        let day = now.day();
        let month = now.month();
        let weekday = now.weekday().num_days_from_monday();

        if !Self::matches_field(minute, parts[0]) {
            return false;
        }
        if !Self::matches_field(hour, parts[1]) {
            return false;
        }
        if !Self::matches_field(day, parts[2]) {
            return false;
        }
        if !Self::matches_field(month, parts[3]) {
            return false;
        }
        if !Self::matches_field(weekday, parts[4]) {
            return false;
        }

        true
    }

    fn matches_field(value: u32, field: &str) -> bool {
        if field == "*" {
            return true;
        }

        if field.starts_with("*/") {
            if let Ok(step) = field[2..].parse::<u32>() {
                return step > 0 && value % step == 0;
            }
        }

        for part in field.split(',') {
            if let Ok(v) = part.parse::<u32>() {
                if v == value {
                    return true;
                }
            }
        }

        false
    }

    fn generate_filename(name: &str) -> String {
        let now = Local::now();
        let day = now.day();
        let month = now.month();
        let hour = now.hour();
        let min = now.minute();
        format!("{}_{:02}{:02}_{:02}{:02}.sql", name, day, month, hour, min)
    }

    fn generate_filepath(backup_dir: &str, connection_name: &str, filename: &str) -> String {
        let now = Local::now();
        let month_name = match now.month() {
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

        format!(
            "{}/{}/{}/{}",
            backup_dir, connection_name, month_name, filename
        )
    }

    fn run_backup(conn: &Connection, filepath: &str) -> Result<(), String> {
        let path = std::path::Path::new(filepath);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let output = Command::new("pg_dump")
            .env("PGPASSWORD", &conn.password)
            .args([
                "-U",
                &conn.user,
                "-h",
                &conn.host,
                "-p",
                &conn.port.to_string(),
                "-d",
                &conn.db_name,
                "-F",
                "p",
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("pg_dump failed: {}", stderr));
        }

        let mut file = std::fs::File::create(filepath).map_err(|e| e.to_string())?;
        file.write_all(&output.stdout).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.stop();
    }
}
