use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::{App, InputMode};
use crate::env_vars::EnvVarType;

pub fn handle_event(app: &mut App, event: &Event) {
    if let Event::Key(key) = event {
        if key.kind != KeyEventKind::Press {
            return;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            app.running = false;
            return;
        }

        match &app.input_mode {
            InputMode::ConfirmQuit => handle_confirm(app, key),
            InputMode::Editing { .. } => handle_editing(app, key),
            InputMode::Normal => handle_normal(app, key),
        }
    }
}

fn handle_confirm(app: &mut App, key: &KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            let _ = app.save();
            app.running = false;
        }
        KeyCode::Char('n') | KeyCode::Char('N') => {
            app.running = false;
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
        }
        _ => {}
    }
}

fn handle_editing(app: &mut App, key: &KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            if let InputMode::Editing { ref buffer, .. } = app.input_mode {
                let value = buffer.clone();
                let name = app.current_var().map(|v| v.name.clone());
                if let Some(name) = name {
                    app.set_value(&name, value);
                }
            }
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Backspace => {
            if let InputMode::Editing { ref mut buffer, ref mut cursor } = app.input_mode {
                if *cursor > 0 {
                    buffer.remove(*cursor - 1);
                    *cursor -= 1;
                }
            }
        }
        KeyCode::Left => {
            if let InputMode::Editing { ref mut cursor, .. } = app.input_mode {
                if *cursor > 0 {
                    *cursor -= 1;
                }
            }
        }
        KeyCode::Right => {
            if let InputMode::Editing { ref buffer, ref mut cursor } = app.input_mode {
                if *cursor < buffer.len() {
                    *cursor += 1;
                }
            }
        }
        KeyCode::Char(c) => {
            if let InputMode::Editing { ref mut buffer, ref mut cursor } = app.input_mode {
                buffer.insert(*cursor, c);
                *cursor += 1;
            }
        }
        _ => {}
    }
}

fn handle_normal(app: &mut App, key: &KeyEvent) {
    match key.code {
        KeyCode::F(2) => {
            let _ = app.save();
            app.running = false;
        }
        KeyCode::Esc => {
            if !app.search.is_empty() {
                app.search.clear();
                app.update_filter();
            } else {
                app.try_quit();
            }
        }
        KeyCode::Backspace => {
            if !app.search.is_empty() {
                app.search.pop();
                app.update_filter();
            }
        }
        KeyCode::Up => {
            if app.var_index > 0 {
                app.var_index -= 1;
                app.desc_scroll = 0;
                if app.var_index < app.var_scroll_offset {
                    app.var_scroll_offset = app.var_index;
                }
            }
        }
        KeyCode::Down => {
            if app.var_index + 1 < app.filtered_count() {
                app.var_index += 1;
                app.desc_scroll = 0;
            }
        }
        KeyCode::Home => {
            app.var_index = 0;
            app.var_scroll_offset = 0;
            app.desc_scroll = 0;
        }
        KeyCode::End => {
            let count = app.filtered_count();
            if count > 0 {
                app.var_index = count - 1;
            }
            app.desc_scroll = 0;
        }
        KeyCode::PageDown => {
            let max = app.filtered_count().saturating_sub(1);
            app.var_index = (app.var_index + 10).min(max);
            app.desc_scroll = 0;
        }
        KeyCode::PageUp => {
            app.var_index = app.var_index.saturating_sub(10);
            if app.var_index < app.var_scroll_offset {
                app.var_scroll_offset = app.var_index;
            }
            app.desc_scroll = 0;
        }
        KeyCode::Left => {
            if app.desc_scroll > 0 {
                app.desc_scroll -= 1;
            }
        }
        KeyCode::Right => {
            app.desc_scroll += 1;
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            if let Some(var) = app.current_var() {
                match var.var_type {
                    EnvVarType::Bool => {
                        let name = var.name.to_string();
                        app.toggle_bool(&name);
                    }
                    EnvVarType::String | EnvVarType::Int => {
                        let current = app.get_value(&var.name).cloned().unwrap_or_default();
                        let cursor = current.len();
                        app.input_mode = InputMode::Editing {
                            buffer: current,
                            cursor,
                        };
                    }
                }
            }
        }
        KeyCode::Delete => {
            if let Some(var) = app.current_var() {
                let name = var.name.to_string();
                app.clear_value(&name);
            }
        }
        KeyCode::Char(c) => {
            app.search.push(c);
            app.update_filter();
        }
        _ => {}
    }
}
