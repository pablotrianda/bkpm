use crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Quit,
    NextTab,
    PrevTab,
    OpenForm,
    EditConnection,
    CloseForm,
    Delete,
    ToggleEnabled,
    FormNextField,
    FormPrevField,
    Submit,
    Backspace,
    InputChar(char),
    ToggleFormEnabled,
    None,
}

pub fn map_key_to_action(code: KeyCode, in_form: bool) -> Action {
    if in_form {
        match code {
            KeyCode::Esc => Action::CloseForm,
            KeyCode::Tab => Action::FormNextField,
            KeyCode::BackTab => Action::FormPrevField,
            KeyCode::Enter => Action::Submit,
            KeyCode::Backspace => Action::Backspace,
            KeyCode::Char('x') => Action::ToggleFormEnabled,
            KeyCode::Char(c) => Action::InputChar(c),
            _ => Action::None,
        }
    } else {
        match code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('n') => Action::OpenForm,
            KeyCode::Char('e') => Action::EditConnection,
            KeyCode::Char('d') => Action::Delete,
            KeyCode::Char('t') => Action::ToggleEnabled,
            KeyCode::Char('x') => Action::ToggleFormEnabled,
            KeyCode::Right | KeyCode::Char('l') => Action::NextTab,
            KeyCode::Left | KeyCode::Char('h') => Action::PrevTab,
            KeyCode::Esc => Action::CloseForm,
            KeyCode::Tab => Action::FormNextField,
            KeyCode::BackTab => Action::FormPrevField,
            KeyCode::Enter => Action::Submit,
            KeyCode::Backspace => Action::Backspace,
            KeyCode::Char(c) => Action::InputChar(c),
            _ => Action::None,
        }
    }
}
