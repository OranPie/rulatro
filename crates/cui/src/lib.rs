mod actions;
mod app;
mod input;
mod persistence;
mod view;

use anyhow::{Context, Result};
use app::{App, UiLocale, DEFAULT_RUN_SEED};
use crossterm::event::{self, Event as CEvent, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, ExecutableCommand};
use persistence::load_auto_perform_file;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, stdout, IsTerminal};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct LaunchOptions {
    pub locale: Option<String>,
    pub seed: Option<u64>,
    pub auto_perform_json: Option<PathBuf>,
}

pub fn run(options: LaunchOptions) -> Result<()> {
    let mut locale_value = options.locale.clone();
    let mut seed_value = options.seed;
    let mut auto_actions = None;
    if let Some(path) = options.auto_perform_json.as_ref() {
        let script = load_auto_perform_file(path)
            .map_err(|err| anyhow::anyhow!(err))
            .with_context(|| format!("load auto perform json from {}", path.display()))?;
        if locale_value.is_none() {
            locale_value = script.locale;
        }
        if seed_value.is_none() {
            seed_value = script.seed;
        }
        auto_actions = Some(script.actions);
    }

    let locale = UiLocale::from_opt(locale_value.as_deref());
    let seed = seed_value.unwrap_or(DEFAULT_RUN_SEED);
    let mut app = App::bootstrap(locale, seed)?;
    if let Some(actions) = auto_actions {
        app.auto_perform_actions(&actions)
            .map_err(|err| anyhow::anyhow!(err))
            .context("apply auto perform actions")?;
    }

    ensure_interactive_terminal()?;

    enable_raw_mode().map_err(|err| {
        anyhow::anyhow!(
            "failed to enable raw mode; ensure the process owns an interactive terminal: {err}"
        )
    })?;
    let mut stdout = stdout();
    stdout
        .execute(EnterAlternateScreen)
        .context("enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("create terminal")?;

    let run_result = run_loop(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    run_result
}

pub fn run_with_args(args: &[String]) -> Result<()> {
    let options = parse_options(args);
    run(options)
}

fn parse_options(args: &[String]) -> LaunchOptions {
    let mut locale = std::env::var("RULATRO_LANG").ok();
    let mut seed = None;
    let mut auto_perform_json = None;
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--lang" | "-l" => {
                if let Some(value) = args.get(idx + 1) {
                    locale = Some(value.clone());
                    idx += 1;
                }
            }
            "--seed" => {
                if let Some(value) = args.get(idx + 1) {
                    seed = value.parse::<u64>().ok();
                    idx += 1;
                }
            }
            "--auto-perform-json" | "--auto-json" => {
                if let Some(value) = args.get(idx + 1) {
                    auto_perform_json = Some(PathBuf::from(value));
                    idx += 1;
                }
            }
            _ => {}
        }
        idx += 1;
    }
    LaunchOptions {
        locale,
        seed,
        auto_perform_json,
    }
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(120);
    while !app.should_quit {
        terminal.draw(|frame| view::draw(frame, app))?;
        if event::poll(tick_rate)? {
            if let CEvent::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if app.handle_path_prompt_key(key) {
                    continue;
                }
                let action = input::map_key(key);
                actions::dispatch(app, action);
            }
        } else {
            app.on_tick();
        }
    }
    Ok(())
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode().context("disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).context("leave alternate screen")?;
    terminal.show_cursor().context("show cursor")?;
    Ok(())
}

fn ensure_interactive_terminal() -> Result<()> {
    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        return Ok(());
    }
    anyhow::bail!(
        "rulatro-cui requires an interactive TTY (run directly in a terminal, not a piped/headless shell)"
    );
}
