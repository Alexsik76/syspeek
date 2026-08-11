use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io::{Result, stdout};
use std::panic;

/// Enables raw mode and the alternate screen on construction, and restores the
/// terminal to its previous state when dropped, so a `?` early return or a
/// panic never leaves the user's shell stuck in raw mode inside the alternate
/// screen.
///
/// Construction also installs a panic hook that restores the terminal before
/// delegating to the previously installed hook. The standard panic handler
/// prints its message to stderr *before* unwinding starts, i.e. while the
/// alternate screen is still active, so without this the message would be
/// printed onto a screen that `Drop` discards moments later.
pub struct TerminalGuard {
    _private: (),
}

impl TerminalGuard {
    pub fn new() -> Result<Self> {
        install_panic_hook();

        enable_raw_mode()?;
        if let Err(err) = stdout().execute(EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(err);
        }
        Ok(Self { _private: () })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // While unwinding from a panic, the hook installed in `new` has
        // already restored the terminal - it runs before unwinding begins.
        // Restoring again here would write a `LeaveAlternateScreen` sequence
        // onto the now-visible screen and corrupt it, so skip it.
        if !std::thread::panicking() {
            restore_terminal();
        }
    }
}

/// Replaces the current panic hook with one that restores the terminal
/// before delegating to the previously installed hook.
fn install_panic_hook() {
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        restore_terminal();
        previous_hook(info);
    }));
}

fn restore_terminal() {
    let _ = stdout().execute(LeaveAlternateScreen);
    let _ = disable_raw_mode();
}
