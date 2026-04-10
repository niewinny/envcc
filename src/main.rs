mod app;
mod env_vars;
mod input;
mod settings;
mod ui;

use std::io;
use std::time::Duration;

use app::App;
use crossterm::event;
use ratatui::DefaultTerminal;

fn main() -> io::Result<()> {
    // Fetch env vars from docs (or cache)
    eprintln!("Fetching environment variables from docs...");
    let fetch = env_vars::fetch_vars().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    if fetch.from_cache {
        eprintln!("Using cached variables ({} vars)", fetch.vars.len());
    } else if fetch.changed {
        eprintln!("Variables updated ({} vars)", fetch.vars.len());
    } else {
        eprintln!("Variables up to date ({} vars)", fetch.vars.len());
    }

    let settings_path = std::env::current_dir()?.join(".claude").join("settings.local.json");
    let (values, other_settings) = settings::load_settings(&settings_path)?;

    let mut app = App::new(fetch.vars, values, other_settings, settings_path);

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();

    result
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    while app.running {
        adjust_scroll(app, terminal.size()?.height);
        terminal.draw(|frame| ui::draw(frame, app))?;

        if event::poll(Duration::from_millis(100))? {
            let ev = event::read()?;
            input::handle_event(app, &ev);
        }

    }

    Ok(())
}

fn adjust_scroll(app: &mut App, terminal_height: u16) {
    // Layout: search(3) + vars(rest) + desc(4) + help(1) = 8 fixed
    // Var list inner = total height - 8 fixed - 2 borders
    let visible = terminal_height.saturating_sub(10) as usize;
    if visible == 0 {
        return;
    }

    if app.var_index >= app.var_scroll_offset + visible {
        app.var_scroll_offset = app.var_index - visible + 1;
    }
    if app.var_index < app.var_scroll_offset {
        app.var_scroll_offset = app.var_index;
    }
}
