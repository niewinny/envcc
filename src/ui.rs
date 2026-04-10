use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, InputMode};
use crate::env_vars::EnvVarType;


pub fn draw(frame: &mut Frame, app: &App) {
    let outer = Layout::vertical([
        Constraint::Length(3), // search bar
        Constraint::Min(5),   // var list
        Constraint::Length(4), // description (2 lines + border)
        Constraint::Length(1), // help bar
    ])
    .split(frame.area());

    draw_search(frame, app, outer[0]);
    draw_variables(frame, app, outer[1]);
    draw_description(frame, app, outer[2]);
    draw_help_bar(frame, app, outer[3]);
}

fn draw_search(frame: &mut Frame, app: &App, area: Rect) {
    let display = if app.search.is_empty() {
        "Type to search...".to_string()
    } else {
        app.search.clone()
    };

    let style = if app.search.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let filtered_info = if !app.search.is_empty() {
        format!(" ({} matches) ", app.filtered_count())
    } else {
        String::new()
    };

    let paragraph = Paragraph::new(display).style(style).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(format!(" Search{}", filtered_info)),
    );

    frame.render_widget(paragraph, area);
}

fn draw_variables(frame: &mut Frame, app: &App, area: Rect) {
    let inner_height = area.height.saturating_sub(2) as usize;

    // Build items with styles for non-selected rows only
    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .enumerate()
        .skip(app.var_scroll_offset)
        .take(inner_height)
        .map(|(fi, &var_idx)| {
            let var = &app.vars[var_idx];
            let value = app.get_value(&var.name);

            // Editing mode for selected item
            if fi == app.var_index {
                if let InputMode::Editing { ref buffer, cursor } = app.input_mode {
                    let mut display_buf = buffer.clone();
                    display_buf.insert(cursor, '|');
                    let edit_display = format!("  {} = [{}]", var.name, display_buf);
                    return ListItem::new(edit_display)
                        .style(Style::default().fg(Color::Indexed(131)).add_modifier(Modifier::BOLD));
                }
            }

            let display = match var.var_type {
                EnvVarType::Bool => {
                    if value.is_some() {
                        format!("[x] {}", var.name)
                    } else {
                        format!("[ ] {}", var.name)
                    }
                }
                EnvVarType::String | EnvVarType::Int => {
                    if let Some(v) = value {
                        format!("  {} = \"{}\"", var.name, v)
                    } else {
                        format!("  {}", var.name)
                    }
                }
            };

            // Non-selected items get their color
            let style = if value.is_some() {
                Style::default().fg(Color::Indexed(131))
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(display).style(style)
        })
        .collect();

    let set_count = app
        .vars
        .iter()
        .filter(|v| app.values.contains_key(&v.name))
        .count();
    let title = format!(
        " envcc - Environment Variables ({}/{} set) ",
        set_count,
        app.vars.len()
    );

    // Use ListState + highlight_style for proper full-width highlight bar
    let selected_in_view = app.var_index.checked_sub(app.var_scroll_offset);

    let list = List::new(items)
        .highlight_style(Style::default().fg(Color::White).bg(Color::Indexed(131)))
        .highlight_symbol("> ")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Indexed(131)))
                .title(title),
        );

    let mut list_state = ListState::default().with_selected(selected_in_view);
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn draw_description(frame: &mut Frame, app: &App, area: Rect) {
    let desc = app
        .current_var()
        .map(|v| v.description.as_str())
        .unwrap_or("");

    let inner_width = area.width.saturating_sub(2) as usize;
    let visible_lines: u16 = area.height.saturating_sub(2);
    let total_lines = if inner_width > 0 && !desc.is_empty() {
        let mut lines = 0u16;
        for line in desc.split('\n') {
            lines += 1 + (line.len() as u16 / inner_width.max(1) as u16);
        }
        lines
    } else {
        0
    };

    let can_scroll_up = app.desc_scroll > 0;
    let can_scroll_down = total_lines > visible_lines + app.desc_scroll;

    let arrows = match (can_scroll_up, can_scroll_down) {
        (true, true) => " \u{2191}\u{2193}",
        (true, false) => " \u{2191}",
        (false, true) => " \u{2193}",
        (false, false) => "",
    };

    let paragraph = Paragraph::new(desc)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false })
        .scroll((app.desc_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(format!(" Description{} ", arrows)),
        );

    frame.render_widget(paragraph, area);
}

fn draw_help_bar(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.input_mode {
        InputMode::ConfirmQuit => {
            " Save changes? y: save & quit  n: quit without saving  Esc: cancel".to_string()
        }
        InputMode::Editing { .. } => {
            " Enter confirm  Esc cancel  type to edit".to_string()
        }
        InputMode::Normal => {
            let dirty_indicator = if app.dirty { "  [modified]" } else { "" };
            format!(
                " Esc quit  F2 save & quit  \u{2191}\u{2193} navigate  \u{2190}\u{2192} desc scroll  Space/Enter toggle/edit  Del clear{}",
                dirty_indicator
            )
        }
    };

    let style = if matches!(app.input_mode, InputMode::ConfirmQuit) {
        Style::default().fg(Color::Indexed(131))
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let paragraph = Paragraph::new(help_text).style(style);

    frame.render_widget(paragraph, area);
}
