use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::state::AppState;

pub fn draw(f: &mut Frame, app: &AppState) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    draw_tabs(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_footer(f, chunks[2]);

    if app.is_form_active() {
        let area = centered_rect(70, 60, size);
        draw_form(f, app, area);
    }
}

fn draw_tabs(f: &mut Frame, app: &AppState, area: Rect) {
    let titles: Vec<Line> = app
        .connections()
        .iter()
        .map(|c| {
            let name = if c.enabled {
                c.name.clone()
            } else {
                format!("{} (disabled)", c.name)
            };
            Line::from(Span::styled(name, Style::default()))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(app.selected_tab())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Connections")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

fn draw_content(f: &mut Frame, app: &AppState, area: Rect) {
    if app.connections().is_empty() {
        let paragraph = Paragraph::new("No connections yet.\n\nPress 'n' to add a new connection.")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Details")
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(paragraph, area);
        return;
    }

    if let Some(conn) = app.current_connection() {
        let content_text = format!(
            "Name: {}\nHost: {}:{}\nUser: {}\nDatabase: {}\nSchedule: {}\nEnabled: {}\nCreated: {}",
            conn.name,
            conn.host,
            conn.port,
            conn.user,
            conn.db_name,
            conn.schedule,
            if conn.enabled { "Yes" } else { "No" },
            conn.created_at
        );

        let paragraph = Paragraph::new(content_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Details")
                .border_style(Style::default().fg(Color::Cyan)),
        );

        f.render_widget(paragraph, area);
    }
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let footer = Paragraph::new(
        "h/l or arrows: navigate | n: new | e: edit | d: delete | t: toggle | q: quit",
    )
    .style(Style::default());

    f.render_widget(footer, area);
}

fn input_create_field<'a>(label: &'a str, value: &'a str, active: bool) -> Line<'a> {
    let style = if active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let display_value = if label == "Password:" && !value.is_empty() {
        "*".repeat(value.len())
    } else {
        value.to_string()
    };

    Line::from(vec![
        Span::raw(format!("{:<12}", label)),
        Span::styled(format!("[ {} ]", display_value), style),
    ])
}

fn draw_form(f: &mut Frame, app: &AppState, area: Rect) {
    let form = &app.form;
    let title = if app.is_edit_mode() {
        "Edit Connection"
    } else {
        "New Connection"
    };

    let enabled_text = if form.enabled { "Yes" } else { "No" };
    let enabled_active = form.selected == 7;

    let lines = vec![
        input_create_field("Name:", &form.name, form.selected == 0),
        input_create_field("Host:", &form.host, form.selected == 1),
        input_create_field("Port:", &form.port, form.selected == 2),
        input_create_field("User:", &form.user, form.selected == 3),
        input_create_field("Password:", &form.password, form.selected == 4),
        input_create_field("Database:", &form.db_name, form.selected == 5),
        input_create_field("Schedule:", &form.schedule, form.selected == 6),
        Line::from(vec![
            Span::raw(format!("{:<12}", "Enabled:")),
            Span::styled(
                format!("[ {} ]", enabled_text),
                if enabled_active {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                },
            ),
            Span::raw(" (press 'x' to toggle)"),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(paragraph, area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
