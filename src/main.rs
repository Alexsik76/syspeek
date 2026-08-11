mod metrics;
mod terminal;

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use metrics::MetricsCollector;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use std::{
    fmt::Write as _,
    io::{Result, stdout},
    sync::mpsc,
    thread,
    time::Duration,
};
use terminal::TerminalGuard;

enum AppEvent {
    Tick,
    Key(KeyEvent),
}

fn is_exit_key(key: KeyEvent) -> bool {
    key.code == KeyCode::Char('q')
        || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
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
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if input_tx.send(AppEvent::Key(key)).is_err() {
                    break;
                }
            }
        }
    });

    let mut collector = MetricsCollector::new();
    let mut metrics = collector.fetch();
    let mut render_buf = String::new();

    terminal.draw(|frame| draw_ui(frame, &metrics, &mut render_buf))?;

    loop {
        match rx.recv() {
            Ok(AppEvent::Key(key)) if is_exit_key(key) => break,
            Ok(AppEvent::Tick) => {
                metrics = collector.fetch();
                terminal.draw(|frame| draw_ui(frame, &metrics, &mut render_buf))?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn draw_ui(frame: &mut Frame, metrics: &metrics::SystemMetrics, render_buf: &mut String) {
    render_buf.clear();
    match metrics.cpu_usage {
        Some(cpu) => {
            let _ = write!(render_buf, "CPU: {cpu:.0}%");
        }
        None => render_buf.push_str("CPU: N/A"),
    }
    let _ = write!(render_buf, "\nRAM: {}", metrics.memory);
    let text = Paragraph::new(render_buf.as_str())
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .title(" Syspeek ")
                .title_bottom(" [q] Exit ")
                .borders(Borders::ALL),
        );
    frame.render_widget(text, frame.area());
}
