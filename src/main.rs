mod metrics;

use metrics::SystemMetrics;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use std::io::{stdout, Result};

fn main() -> Result<()> {
    // 1. Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // 1.1. Get data
    let metrics = SystemMetrics::fetch();

    // 2. Draw UI
    terminal.draw(|frame| {
        let content = format!("CPU: {}%\nRAM: {}", metrics.cpu_usage, metrics.memory_usage);
        let text = Paragraph::new(content)
            .style(Style::default().fg(Color::Cyan))
            .block(
                Block::default()
                    .title(" Syspeek ")
                    .title_bottom(" [q] Exit ")
                    .borders(Borders::ALL)
            );
        frame.render_widget(text, frame.area());
    })?;

    // 3. Wait for 'q' key to exit
    loop {
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break;
            }
        }
    }

    // 4. Restore terminal state
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}