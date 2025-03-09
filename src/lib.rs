#![deny(clippy::all)]

// 🤓☝️
// 找了半天，竟然没有人做这个
// 一半的代码都是参考大模型的，我对Rust的学习并不深入

#[macro_use]
extern crate napi_derive;

mod media_control;
mod monitor;
mod session_manager;
mod types;
mod utils;

pub use crate::media_control::{get_all_sessions, get_current_session, get_session_by_id};
pub use crate::monitor::SMTCMonitor;
pub use crate::types::{MediaInfo, MediaProps, PlaybackInfo, TimelineProps};
