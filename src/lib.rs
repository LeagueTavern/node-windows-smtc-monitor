#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

mod media_control;
mod monitor;
mod session_manager;
mod types;
mod utils;

pub use crate::media_control::{get_sessions, get_current_session, get_session_by_id};
pub use crate::monitor::SMTCMonitor;
pub use crate::types::{MediaInfo, MediaProps, PlaybackInfo, TimelineProps};
