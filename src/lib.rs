#![deny(clippy::all)]

// 🤓☝️
// 找了半天，竟然没有人做这个，那就自己写一个

#[macro_use]
extern crate napi_derive;

mod media_control;
mod monitor;
mod session_manager;
mod types;
mod utils;

// 从各个模块重导出公共API
pub use crate::media_control::{get_all_sessions, get_current_session, get_session_by_id};
pub use crate::monitor::SMTCMonitor;
pub use crate::types::MediaInfo;
