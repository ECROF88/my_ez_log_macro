use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::thread::{self, JoinHandle};

use crossbeam::channel::{Receiver, Sender, unbounded};
use lazy_static::lazy_static;
use std::cell::RefCell;

pub struct LogConfig {
    file_path: PathBuf,
}
impl Default for LogConfig {
    fn default() -> Self {
        Self {
            file_path: PathBuf::from("loginfo.log"),
        }
    }
}

// lazy_static! {
//     static ref LOG_CONFIG: Mutex<LogConfig> = Mutex::new(LogConfig::default());
// }
static LOG_FILE_PATH: OnceLock<PathBuf> = OnceLock::new();
// pub fn init_logging(file_name: &str) {
//     let mut config = LOG_CONFIG.lock().unwrap();
//     config.file_path = PathBuf::from(file_name); // 创建一个拥有所有权的路径
// }
pub fn init_logging(file_name: &str) {
    let _ = LOG_FILE_PATH.set(PathBuf::from(file_name));
}

/// 日志级别
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

pub enum LogMessage {
    StaticStr(&'static str),
    LazyFormat(Box<dyn Fn() -> String + Send>),
}

/// 日志条目结构体
pub struct LogEntry {
    pub level: LogLevel,
    pub message: LogMessage,
    pub timestamp: u64,
}

impl LogEntry {
    pub fn format(&self) -> String {
        let formatted_time = self.timestamp;
        let message = match &self.message {
            LogMessage::StaticStr(s) => s.to_string(),
            LogMessage::LazyFormat(f) => f(),
        };

        format!("[{}][{:?}] {}", formatted_time, self.level, message)
    }
}

// THREAD_BUFFER
thread_local! {
    static THREAD_BUFFER: RefCell<Vec<LogEntry>> = RefCell::new(Vec::with_capacity(128));
}

pub fn buffer_log(entry: LogEntry) {
    let should_flush = THREAD_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        // println!("get entry: {}", entry.format());
        buffer.push(entry);
        buffer.len() >= 10
    });
    if should_flush {
        // println!("go to flush");
        write_all_and_flush();
    }
}
pub fn write_all_and_flush() {
    THREAD_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        for entry in buffer.drain(..) {
            let _ = GLOBAL_SENDER.send(entry);
        }
    })
}
lazy_static! {
    pub static ref GLOBAL_SENDER: Sender<LogEntry> = {
        let (sender, receiver) = unbounded();
        // 启动后台写入线程
        let _handle = start_background_writer(receiver);
        sender
    };
}
#[macro_export]
macro_rules! log {
    // 处理静态字符串：nanolog!(LogLevel::Info, "message")
    ($level:expr, $msg:expr) => {{
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        $crate::log::buffer_log($crate::log::LogEntry {
                level: $level,
                message: $crate::log::LogMessage::StaticStr($msg),
                timestamp,
            })
    }};

    // 处理格式化字符串：nanolog!(LogLevel::Info, "User {} login {}", user, count)
    ($level:expr, $fmt:expr, $($arg:expr),*) => {{
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let formatter = Box::new(move || {
            format!($fmt, $($arg),*)
        });

        $crate::log::buffer_log($crate::log::LogEntry {
                level: $level,
                message: $crate::log::LogMessage::LazyFormat(formatter),
                timestamp,
            })
    }};
}

// ----------------------------
// 后台写入线程
// ----------------------------
fn start_background_writer(receiver: Receiver<LogEntry>) -> JoinHandle<()> {
    thread::spawn(move || {
        // let file = &LOG_CONFIG.lock().unwrap().file_path;
        let file = LOG_FILE_PATH.get().expect("fail to get file path");
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(file)
            .unwrap();

        for entry in receiver {
            let formatted = entry.format();
            // println!("写入文件！");
            writeln!(
                file,
                "[{}][{:?}] {}",
                entry.timestamp, entry.level, formatted
            )
            .unwrap();
        }
    })
}
pub fn shutdown_logging() {
    write_all_and_flush();
    std::thread::sleep(std::time::Duration::from_millis(50));
}
