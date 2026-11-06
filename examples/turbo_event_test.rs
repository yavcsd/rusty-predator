//! Launch Manager experiment: set EC bit `LMIB` with `APGeAction.SetFunction`, log every
//! `APGeEvent` and Turbo flag change for 30 s while the user presses Turbo, then clear the bit.
//! Waits for Rusty Predator to close first, because it reacts to the same button.
//!
//! Run elevated: `turbo_event_test.exe <report-file>`
//! If the window was closed early, clear the bit with: `turbo_event_test.exe off`

mod common;

use common::{capture_for_30s, log, save_log};
use rusty_predator::controller::Backend;
use rusty_predator::protocol::{Method, MiscSetting, LAUNCH_MANAGER_OFF as LM_OFF, LAUNCH_MANAGER_ON as LM_ON};
use rusty_predator::wmi::{init_security, EventStream, Wmi};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use windows::core::BOOL;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
use windows::Win32::System::Console::SetConsoleCtrlHandler;

/// Report path, for the close handler.
static REPORT: OnceLock<String> = OnceLock::new();

/// Window closed or Ctrl+C: clear the bit and save the log before Windows ends the process.
unsafe extern "system" fn on_close(_: u32) -> BOOL {
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    let r = Wmi::connect().map_or_else(|e| e.to_string(), |hw| raw(&hw, Method::ApgeSetFunction, LM_OFF));
    log(&format!("window closed early: LM off -> {r}"));
    if let Some(p) = REPORT.get() {
        save_log(p);
    }
    BOOL(0)
}

const APP_CLOSE_WAIT: Duration = Duration::from_secs(120);

/// Log a line and show it in the console.
fn say(s: &str) {
    println!("{s}");
    log(s);
}

fn raw(hw: &Wmi, m: Method, input: u64) -> String {
    hw.call(m, input).map_or_else(|e| e.to_string(), |v| format!("{v:#x}"))
}

fn turbo(hw: &Wmi) -> String {
    raw(hw, Method::GetGamingMiscSetting, MiscSetting::Turbo as u64)
}

/// A failed `tasklist` counts as running, so the test never starts next to the app.
fn app_running() -> bool {
    std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq rusty-predator.exe", "/NH"])
        .output()
        .map_or(true, |o| String::from_utf8_lossy(&o.stdout).contains("rusty-predator"))
}

fn spawn_event_log() {
    std::thread::spawn(|| unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        match EventStream::open() {
            Ok(es) => {
                say("subscribed to APGeEvent");
                loop {
                    match es.next() {
                        Ok(d) => say(&format!("EVENT APGeEvent {d:02X?}")),
                        Err(e) => {
                            say(&format!("event error: {e}"));
                            return;
                        }
                    }
                }
            }
            Err(e) => say(&format!("subscribe failed: {e}")),
        }
    });
}

/// Shows that a press registered even if no event arrives.
fn spawn_turbo_watch() {
    std::thread::spawn(|| unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let Ok(hw) = Wmi::connect() else { return };
        let mut last = turbo(&hw);
        loop {
            std::thread::sleep(Duration::from_millis(250));
            let now = turbo(&hw);
            if now != last {
                say(&format!("Turbo flag changed: {last} -> {now}"));
                last = now;
            }
        }
    });
}

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_else(|| "turbo_event_test.txt".into());
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    init_security();
    let hw = match Wmi::connect() {
        Ok(h) => h,
        Err(e) => {
            println!("connect failed: {e}");
            if arg != "off" {
                let _ = std::fs::write(&arg, format!("connect failed: {e}\n"));
            }
            return;
        }
    };
    if arg == "off" {
        println!("off -> {}", raw(&hw, Method::ApgeSetFunction, LM_OFF));
        return;
    }

    println!("Don't close this window: it closes by itself in about a minute.");
    let _ = REPORT.set(arg.clone());
    let _ = unsafe { SetConsoleCtrlHandler(Some(on_close), true) };
    let t0 = Instant::now();
    if app_running() {
        println!("Close Rusty Predator from its tray icon (Exit). This test starts once it has closed.");
    }
    while app_running() {
        if t0.elapsed() > APP_CLOSE_WAIT {
            let _ = std::fs::write(&arg, "not started: Rusty Predator stayed open, nothing was changed\n");
            return;
        }
        std::thread::sleep(Duration::from_secs(1));
    }

    spawn_event_log();
    std::thread::sleep(Duration::from_secs(1));
    let start = turbo(&hw);
    say(&format!("start: Turbo {start}, GetFunction(1) {}", raw(&hw, Method::ApgeGetFunction, 1)));
    let on = hw.call(Method::ApgeSetFunction, LM_ON);
    say(&format!("LM on -> {}", on.as_ref().map_or_else(ToString::to_string, |v| format!("{v:#x}"))));
    say(&format!("after on: Turbo {}, GetFunction(1) {}", turbo(&hw), raw(&hw, Method::ApgeGetFunction, 1)));

    if on.is_ok() {
        spawn_turbo_watch();
        capture_for_30s("30 s: press Turbo, wait 5 s, press Turbo again. Then press Fn + brightness up once.");
        println!("capture finished");
    }

    // Runs after a failed set too: clearing the bit is harmless and is its original state.
    say(&format!("LM off -> {}", raw(&hw, Method::ApgeSetFunction, LM_OFF)));
    let end = turbo(&hw);
    say(&format!("end: Turbo {end}, GetFunction(1) {}", raw(&hw, Method::ApgeGetFunction, 1)));
    if end != start {
        say("Turbo differs from the start: press the button once more to return.");
    }
    save_log(&arg);
    println!("Done. This window closes in 8 s.");
    std::thread::sleep(Duration::from_secs(8));
}
