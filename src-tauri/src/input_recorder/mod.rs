use flate2::write::GzEncoder;
use flate2::Compression;
use serde::Serialize;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

pub(crate) struct RawInputEvent {
    pub time: SystemTime,
    pub event_type: &'static str, // "kd", "ku", "bd", "bu"
    pub key: String,
    pub raw: u32,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
struct InputEvent {
    t: u64,
    #[serde(rename = "type")]
    event_type: &'static str,
    key: String,
    raw: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

fn system_time_millis(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub struct InputRecorder {
    recording: Arc<AtomicBool>,
    writer_handle: Option<std::thread::JoinHandle<()>>,
    listener_handle: Option<std::thread::JoinHandle<()>>,
    #[cfg(target_os = "macos")]
    run_loop: Option<macos::RunLoopRef>,
    #[cfg(windows)]
    hwnd: Option<windows::SendHwnd>,
}

impl InputRecorder {
    pub fn new(session_path: &Path) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(session_path)?;
        let writer = BufWriter::new(file);

        let recording = Arc::new(AtomicBool::new(true));
        let (tx, rx) = mpsc::channel::<RawInputEvent>();

        let recording_clone = Arc::clone(&recording);
        let writer_handle = std::thread::spawn(move || {
            Self::writer_loop(rx, writer, recording_clone);
        });

        #[cfg(target_os = "macos")]
        let (listener_handle, run_loop) = {
            let (handle, rl) = macos::start_listener(tx, Arc::clone(&recording));
            (handle, Some(rl))
        };

        #[cfg(windows)]
        let (listener_handle, hwnd) = {
            let (handle, h) = windows::start_listener(tx, Arc::clone(&recording));
            (handle, Some(h))
        };

        Ok(Self {
            recording,
            writer_handle: Some(writer_handle),
            listener_handle: Some(listener_handle),
            #[cfg(target_os = "macos")]
            run_loop,
            #[cfg(windows)]
            hwnd,
        })
    }

    fn writer_loop(
        rx: mpsc::Receiver<RawInputEvent>,
        mut writer: BufWriter<File>,
        recording: Arc<AtomicBool>,
    ) {
        let mut event_count = 0u64;
        let flush_interval = 100;
        let mut keys_held: HashSet<String> = HashSet::new();

        while recording.load(Ordering::Relaxed) {
            match rx.recv_timeout(std::time::Duration::from_secs(5)) {
                Ok(event) => {
                    Self::write_event(event, &mut writer, &mut keys_held, &mut event_count, flush_interval);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    let _ = writer.flush();
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        // Drain any events still queued in the channel after the recording
        // flag was set to false but before the listener thread fully stopped.
        for event in rx.try_iter() {
            Self::write_event(event, &mut writer, &mut keys_held, &mut event_count, flush_interval);
        }

        let _ = writer.flush();
    }

    fn write_event(
        event: RawInputEvent,
        writer: &mut BufWriter<File>,
        keys_held: &mut HashSet<String>,
        event_count: &mut u64,
        flush_interval: u64,
    ) {
        match event.event_type {
            "kd" | "bd" => {
                if !keys_held.insert(event.key.clone()) {
                    return;
                }
            }
            "ku" | "bu" => {
                keys_held.remove(&event.key);
            }
            _ => {}
        }

        let input_event = InputEvent {
            t: system_time_millis(event.time),
            event_type: event.event_type,
            key: event.key,
            raw: event.raw,
            name: event.name,
        };
        if let Ok(json) = serde_json::to_string(&input_event) {
            let _ = writeln!(writer, "{}", json);
            *event_count += 1;
            if *event_count % flush_interval == 0 {
                let _ = writer.flush();
            }
        }
    }

    pub fn stop(&mut self) {
        self.recording.store(false, Ordering::SeqCst);

        #[cfg(target_os = "macos")]
        if let Some(rl) = self.run_loop.take() {
            rl.stop();
        }

        #[cfg(windows)]
        if let Some(hwnd) = self.hwnd.take() {
            hwnd.close();
        }

        if let Some(handle) = self.listener_handle.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.writer_handle.take() {
            let _ = handle.join();
        }
    }
}

pub fn gzip_file(input_path: &Path) -> std::io::Result<PathBuf> {
    let output_path = input_path.with_extension("jsonl.gz");
    let input_file = File::open(input_path)?;
    let output_file = File::create(&output_path)?;
    let mut encoder = GzEncoder::new(output_file, Compression::default());
    std::io::copy(&mut std::io::BufReader::new(input_file), &mut encoder)?;
    encoder.finish()?;
    Ok(output_path)
}

#[cfg(target_os = "macos")]
pub fn check_accessibility_permission() -> bool {
    macos_accessibility_client::accessibility::application_is_trusted_with_prompt()
}

#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permission() -> bool {
    true
}
