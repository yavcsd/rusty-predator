#![cfg_attr(not(test), windows_subsystem = "windows")]

//! Rusty Predator: a small replacement for Acer PredatorSense.
//!
//! Usage:
//!   rusty-predator                 open the window (and tray icon)
//!   rusty-predator --hidden        start in the tray only
//!   rusty-predator --toggle        show/hide a running instance, or start with the window open
//!   rusty-predator --status [FILE] read-only hardware report to FILE (or stdout; but the
//!                                 elevation prompt starts a new process, so stdout is only
//!                                 seen from an already elevated prompt: pass FILE)
//!
//! Needs administrator rights (the BIOS WMI methods refuse other callers); the manifest that
//! build.rs embeds requires elevation. Everything else lives in the library crate.

use rusty_predator::gui::{self, StartMode};
use rusty_predator::{status_report, wmi};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};

/// `--status`: write the report; exit code 0 on success, 1 if WMI or the write failed.
fn status_mode(out: Option<&str>) -> i32 {
    let text = match wmi::Wmi::connect() {
        Ok(w) => status_report::report(w).0,
        Err(e) => format!("error: {e}\n"),
    };
    let ok = if let Some(path) = out {
        std::fs::write(path, &text).is_ok()
    } else {
        use std::io::Write;
        std::io::stdout().write_all(text.as_bytes()).is_ok()
    };
    i32::from(!ok || text.starts_with("error"))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // The window and WMI calls share the main (STA) thread; the hotkey thread has its own MTA.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
    wmi::init_security();
    let code = match args.first().map(String::as_str) {
        Some("--status") => status_mode(args.get(1).map(String::as_str)),
        Some("--hidden") => gui::run(StartMode::Hidden),
        Some("--toggle") => gui::run(StartMode::Toggle),
        None => gui::run(StartMode::Window),
        Some(_) => {
            eprintln!("usage: rusty-predator [--hidden | --toggle | --status [FILE]]");
            2
        }
    };
    std::process::exit(code);
}
