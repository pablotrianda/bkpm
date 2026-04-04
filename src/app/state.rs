use super::events::Action;
use crate::db::Db;
use crate::models::Connection;
use crate::scheduler::Scheduler;

pub struct AppState {
    mode: Mode,
    connections: Vec<Connection>,
    selected_tab: usize,
    pub form: FormState,
    pub db: Db,
    pub scheduler: Scheduler,
    pub backup_dir: String,
}

pub enum Mode {
    Normal,
    Form,
    Edit,
}

pub struct FormState {
    pub id: Option<i64>,
    pub name: String,
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub db_name: String,
    pub schedule: String,
    pub selected: usize,
    pub enabled: bool,
}

impl FormState {
    pub fn new() -> Self {
        Self {
            id: None,
            name: String::new(),
            host: String::new(),
            port: String::from("5432"),
            user: String::new(),
            password: String::new(),
            db_name: String::new(),
            schedule: String::from("0 2 * * *"),
            selected: 0,
            enabled: true,
        }
    }

    pub fn from_connection(conn: &Connection) -> Self {
        Self {
            id: Some(conn.id),
            name: conn.name.clone(),
            host: conn.host.clone(),
            port: conn.port.to_string(),
            user: conn.user.clone(),
            password: conn.password.clone(),
            db_name: conn.db_name.clone(),
            schedule: conn.schedule.clone(),
            selected: 0,
            enabled: conn.enabled,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "bkpm.db".to_string());
        let db = Db::new(&db_path).expect("Failed to open database");
        let connections = db.get_all().unwrap_or_default();

        let backup_dir = std::env::var("BACKUP_DIR").unwrap_or_else(|_| "./backups".to_string());
        let mut scheduler = Scheduler::new();
        scheduler.start(connections.clone(), backup_dir.clone());

        Self {
            connections,
            selected_tab: 0,
            form: FormState::new(),
            mode: Mode::Normal,
            db,
            scheduler,
            backup_dir,
        }
    }

    pub fn next(&mut self) {
        if !self.connections.is_empty() && self.selected_tab < self.connections.len() - 1 {
            self.selected_tab += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.selected_tab > 0 {
            self.selected_tab -= 1;
        }
    }

    pub fn selected_tab(&self) -> usize {
        self.selected_tab
    }

    pub fn current_connection(&self) -> Option<&Connection> {
        self.connections.get(self.selected_tab)
    }

    pub fn connections(&self) -> &[Connection] {
        &self.connections
    }

    pub fn is_normal_mode(&self) -> bool {
        matches!(&self.mode, Mode::Normal)
    }

    pub fn set_normal_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn set_form_mode(&mut self) {
        self.form = FormState::new();
        self.mode = Mode::Form;
    }

    pub fn set_edit_mode(&mut self) {
        if let Some(conn) = self.current_connection() {
            self.form = FormState::from_connection(conn);
            self.mode = Mode::Edit;
        }
    }

    pub fn is_form_active(&self) -> bool {
        matches!(self.mode, Mode::Form | Mode::Edit)
    }

    pub fn is_edit_mode(&self) -> bool {
        matches!(self.mode, Mode::Edit)
    }

    pub fn update(&mut self, action: Action) -> bool {
        let in_form = self.is_form_active();

        match action {
            Action::OpenForm if !in_form => self.set_form_mode(),
            Action::EditConnection if !in_form && self.is_normal_mode() => self.set_edit_mode(),
            Action::CloseForm => {
                self.set_normal_mode();
            }
            Action::NextTab if !in_form && self.is_normal_mode() => self.next(),
            Action::PrevTab if !in_form && self.is_normal_mode() => self.previous(),
            Action::ToggleEnabled if !in_form && self.is_normal_mode() => {
                if let Some(conn) = self.current_connection() {
                    let _ = self.db.toggle_enabled(conn.id);
                    self.connections = self.db.get_all().unwrap_or_default();
                }
            }
            Action::Delete if !in_form && self.is_normal_mode() => {
                if let Some(conn) = self.current_connection() {
                    let _ = self.db.delete(conn.id);
                    self.connections = self.db.get_all().unwrap_or_default();
                    if self.selected_tab >= self.connections.len() && !self.connections.is_empty() {
                        self.selected_tab = self.connections.len() - 1;
                    }
                }
            }
            Action::Quit if !in_form => return true,
            Action::Submit if self.is_form_active() => {
                let form = &self.form;
                let conn = Connection::new(
                    form.name.clone(),
                    form.host.clone(),
                    form.port.parse().unwrap_or(5432),
                    form.user.clone(),
                    form.password.clone(),
                    form.db_name.clone(),
                    form.schedule.clone(),
                );

                if self.is_edit_mode() {
                    let mut conn = conn;
                    conn.id = form.id.unwrap();
                    conn.enabled = form.enabled;
                    let _ = self.db.update(&conn);
                } else {
                    let _ = self.db.insert(&conn);
                }

                self.connections = self.db.get_all().unwrap_or_default();
                self.set_normal_mode();
            }
            _ => {}
        }

        false
    }

    pub fn update_form(&mut self, action: Action) {
        let form = &mut self.form;
        match action {
            Action::FormNextField => {
                form.selected = (form.selected + 1) % 8;
            }
            Action::FormPrevField => {
                if form.selected == 0 {
                    form.selected = 7;
                } else {
                    form.selected -= 1;
                }
            }
            Action::InputChar(c) => match form.selected {
                0 => form.name.push(c),
                1 => form.host.push(c),
                2 => form.port.push(c),
                3 => form.user.push(c),
                4 => form.password.push(c),
                5 => form.db_name.push(c),
                6 => form.schedule.push(c),
                _ => {}
            },
            Action::Backspace => match form.selected {
                0 => {
                    form.name.pop();
                }
                1 => {
                    form.host.pop();
                }
                2 => {
                    form.port.pop();
                }
                3 => {
                    form.user.pop();
                }
                4 => {
                    form.password.pop();
                }
                5 => {
                    form.db_name.pop();
                }
                6 => {
                    form.schedule.pop();
                }
                _ => {}
            },
            Action::ToggleFormEnabled => {
                form.enabled = !form.enabled;
            }
            _ => {}
        }
    }
}
