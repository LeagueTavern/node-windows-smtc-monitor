use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use std::collections::HashMap;
use windows::Foundation::TypedEventHandler;
use windows::Media::Control::GlobalSystemMediaTransportControlsSession;

use crate::types::MediaInfo;
use crate::utils;

#[allow(dead_code)]
pub struct InnerSession {
  pub session: GlobalSystemMediaTransportControlsSession,
  pub callbacks: Vec<ThreadsafeFunction<String>>,
}

pub struct SessionManager {
  pub sessions: HashMap<String, InnerSession>,
  pub session_added_callbacks: Vec<ThreadsafeFunction<MediaInfo>>,
  pub session_removed_callbacks: Vec<ThreadsafeFunction<String>>,
  pub media_props_callbacks: Vec<ThreadsafeFunction<MediaInfo>>,
  pub playback_info_callbacks: Vec<ThreadsafeFunction<MediaInfo>>,
  pub timeline_props_callbacks: Vec<ThreadsafeFunction<MediaInfo>>,
}

impl SessionManager {
  pub fn new() -> Self {
    Self {
      sessions: HashMap::new(),
      session_added_callbacks: Vec::new(),
      session_removed_callbacks: Vec::new(),
      media_props_callbacks: Vec::new(),
      playback_info_callbacks: Vec::new(),
      timeline_props_callbacks: Vec::new(),
    }
  }
}

// 注册会话方法
pub fn register_session(
  inner: &mut SessionManager,
  id: String,
  session: GlobalSystemMediaTransportControlsSession,
) {
  // 创建独立的会话克隆
  // 为每个回调单独克隆会话对象和回调列表
  let media_session_clone = session.clone();
  let media_props_callbacks = inner.media_props_callbacks.clone();
  let playback_session_clone = session.clone();
  let playback_info_callbacks = inner.playback_info_callbacks.clone();
  let timeline_session_clone = session.clone();
  let timeline_props_callbacks = inner.timeline_props_callbacks.clone();

  // 媒体属性改变事件
  let _media_token = session.MediaPropertiesChanged(&TypedEventHandler::new(move |_, _| {
    // 尝试获取最新的 MediaInfo
    if let Ok(Some(media_info)) = utils::get_media_info_for_session(&media_session_clone) {
      for callback in &media_props_callbacks {
        callback.call(Ok(media_info.clone()), ThreadsafeFunctionCallMode::Blocking);
      }
    }
    Ok(())
  }));

  // 播放信息改变事件
  let _playback_token = session.PlaybackInfoChanged(&TypedEventHandler::new(move |_, _| {
    // 尝试获取最新的 MediaInfo
    if let Ok(Some(media_info)) = utils::get_media_info_for_session(&playback_session_clone) {
      for callback in &playback_info_callbacks {
        callback.call(Ok(media_info.clone()), ThreadsafeFunctionCallMode::Blocking);
      }
    }
    Ok(())
  }));

  // 时间线改变事件
  let _timeline_token = session.TimelinePropertiesChanged(&TypedEventHandler::new(move |_, _| {
    // 尝试获取最新的 MediaInfo
    if let Ok(Some(media_info)) = utils::get_media_info_for_session(&timeline_session_clone) {
      for callback in &timeline_props_callbacks {
        callback.call(Ok(media_info.clone()), ThreadsafeFunctionCallMode::Blocking);
      }
    }
    Ok(())
  }));

  // 将会话保存到内部状态
  inner.sessions.insert(
    id.clone(),
    InnerSession {
      session: session.clone(),
      callbacks: Vec::new(),
    },
  );

  // 通知会话已添加，并传递完整的 MediaInfo
  // 尝试获取 MediaInfo
  if let Ok(Some(media_info)) = utils::get_media_info_for_session(&session) {
    for callback in &inner.session_added_callbacks {
      callback.call(Ok(media_info.clone()), ThreadsafeFunctionCallMode::Blocking);
    }
  }
}
