use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tokio::sync::watch;
use tokio::time::{interval, Duration};

const POLL_INTERVAL_MS: u64 = 500;

#[derive(Default)]
pub struct FocusFlag(AtomicBool);

impl FocusFlag {
    fn store(&self, v: bool) {
        self.0.store(v, Ordering::SeqCst);
    }
    fn load(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

pub fn init(app: &tauri::AppHandle) -> watch::Receiver<bool> {
    let flag = Arc::new(FocusFlag::default());
    app.manage(flag.clone());
    start_poller(flag)
}

pub fn is_focused(app: &tauri::AppHandle) -> bool {
    app.state::<Arc<FocusFlag>>().load()
}

fn start_poller(flag: Arc<FocusFlag>) -> watch::Receiver<bool> {
    let (tx, rx) = watch::channel(false);

    tauri::async_runtime::spawn(async move {
        let mut tick = interval(Duration::from_millis(POLL_INTERVAL_MS));
        let cached_pid: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));

        loop {
            tick.tick().await;
            let cache = cached_pid.clone();
            let focused = tokio::task::spawn_blocking(move || is_game_focused_blocking(&cache))
                .await
                .unwrap_or(false);

            flag.store(focused);

            let _ = tx.send_if_modified(|current| {
                if *current != focused {
                    log::info!("game focus changed: focused={}", focused);
                    *current = focused;
                    true
                } else {
                    false
                }
            });

            if tx.is_closed() {
                break;
            }
        }
    });

    rx
}

#[allow(unused_variables)]
fn is_game_focused_blocking(cached_pid: &Mutex<Option<u32>>) -> bool {
    #[cfg(target_os = "windows")]
    {
        is_hots_foreground_windows(cached_pid)
    }
    #[cfg(not(target_os = "windows"))]
    {
        // macOS HoTS isn't realistically in scope; fall back to "is the
        // process running" — matches the existing is_game_running() check.
        crate::is_game_running()
    }
}

#[cfg(target_os = "windows")]
fn is_hots_foreground_windows(cached_pid: &Mutex<Option<u32>>) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId,
    };

    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return false;
    }

    let mut foreground_pid: u32 = 0;
    let _thread_id =
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut foreground_pid as *mut u32)) };
    if foreground_pid == 0 {
        return false;
    }

    {
        let cached = cached_pid.lock().unwrap();
        if let Some(pid) = *cached {
            if pid == foreground_pid {
                return true;
            }
        }
    }

    let hots_pid = lookup_hots_pid_windows();
    *cached_pid.lock().unwrap() = hots_pid;
    hots_pid == Some(foreground_pid)
}

#[cfg(target_os = "windows")]
fn lookup_hots_pid_windows() -> Option<u32> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let output = std::process::Command::new("tasklist")
        .args([
            "/NH",
            "/FO",
            "CSV",
            "/FI",
            "IMAGENAME eq HeroesOfTheStorm_x64.exe",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() >= 2 {
            let pid_str = cols[1].trim().trim_matches('"');
            if let Ok(pid) = pid_str.parse::<u32>() {
                return Some(pid);
            }
        }
    }
    None
}
