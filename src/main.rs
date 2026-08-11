mod metrics;
mod terminal;

use crossterm::event::{self, Event as CEvent, KeyCode};
use metrics::MetricsCollector;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use std::{
    io::{Result, stdout},
    sync::mpsc,
    thread,
    time::Duration,
};
use terminal::TerminalGuard;

enum AppEvent {
    Tick,
    Key(KeyCode),
}

fn main() -> Result<()> {
    let _terminal_guard = TerminalGuard::new()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let (tx, rx) = mpsc::channel();

    let tick_tx = tx.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            if tick_tx.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });

    let input_tx = tx.clone();
    thread::spawn(move || {
        loop {
            if let Ok(CEvent::Key(key)) = event::read() {
                if input_tx.send(AppEvent::Key(key.code)).is_err() {
                    break;
                }
            }
        }
    });

    let mut collector = MetricsCollector::new();
    let mut metrics = collector.fetch();

    terminal.draw(|frame| draw_ui(frame, &metrics))?;

    loop {
        match rx.recv() {
            Ok(AppEvent::Key(KeyCode::Char('q'))) => break,
            Ok(AppEvent::Tick) => {
                metrics = collector.fetch();
                terminal.draw(|frame| draw_ui(frame, &metrics))?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn draw_ui(frame: &mut Frame, metrics: &metrics::SystemMetrics) {
    let content = format!("CPU: {}%\nRAM: {}", metrics.cpu_usage, metrics.memory_usage);
    let text = Paragraph::new(content)
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .title(" Syspeek ")
                .title_bottom(" [q] Exit ")
                .borders(Borders::ALL),
        );
    frame.render_widget(text, frame.area());
}
