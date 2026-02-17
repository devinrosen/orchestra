use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use rusqlite::{Connection, OpenFlags};

mod app;
mod db;
mod player;
mod ui;

use app::App;

fn main() {
    // Parse optional --db <path> CLI flag.
    let db_path: PathBuf = {
        let mut args = std::env::args().skip(1);
        let mut override_path: Option<PathBuf> = None;
        while let Some(arg) = args.next() {
            if arg == "--db" {
                if let Some(p) = args.next() {
                    override_path = Some(PathBuf::from(p));
                }
            }
        }
        if let Some(p) = override_path {
            p
        } else {
            match db::find_db_path() {
                Some(p) => p,
                None => {
                    eprintln!(
                        "Error: Orchestra database not found.\n\
                         Expected location: ~/Library/Application Support/com.orchestra.app/orchestra.db\n\
                         Use --db <path> to specify an alternative location.\n\
                         Please launch the Orchestra desktop app and scan a library first."
                    );
                    std::process::exit(1);
                }
            }
        }
    };

    // Open the database in read-only mode.
    let conn = match Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: Failed to open database at {}: {}", db_path.display(), e);
            std::process::exit(1);
        }
    };

    // Read library root from settings.
    let library_root = match db::read_library_root(&conn) {
        Ok(Some(root)) => root,
        Ok(None) => {
            eprintln!(
                "Error: No library root found in the database.\n\
                 Please open the Orchestra desktop app and scan a music library first."
            );
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: Failed to read settings: {e}");
            std::process::exit(1);
        }
    };

    // Load the full library tree.
    let tree = match orchestra_core::db::library_repo::get_library_tree(&conn, &library_root) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error: Failed to load library: {e}");
            std::process::exit(1);
        }
    };

    if tree.total_tracks == 0 {
        eprintln!(
            "Warning: Library is empty (0 tracks in \"{library_root}\").\n\
             Please scan a music library in the Orchestra desktop app first."
        );
        // Continue anyway — the UI will show empty panes.
    }

    // Initialize terminal.
    if let Err(e) = run_tui(tree) {
        eprintln!("TUI error: {e}");
        std::process::exit(1);
    }
}

fn run_tui(tree: orchestra_core::models::track::LibraryTree) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(tree);

    let result = event_loop(&mut terminal, &mut app);

    // Always restore terminal state, even on error.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    app.player.stop();

    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        // Drain player errors into status_msg.
        app.tick();

        // Draw frame.
        terminal.draw(|f| ui::draw(f, app))?;

        // Poll for keyboard events (~60 fps).
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key-press events (not repeat or release on Windows).
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
