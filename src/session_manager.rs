use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use std::collections::HashMap;
use windows::Foundation::{EventRegistrationToken, TypedEventHandler};
use windows::Media::Control::GlobalSystemMediaTransportControlsSession;

use crate::monitor::{MediaPropsCallbackData, PlaybackInfoCallbackData, TimelinePropsCallbackData};
use crate::types::MediaInfo;
use crate::utils;

#[allow(dead_code)]
pub struct InnerSession {
  pub session: GlobalSystemMediaTransportControlsSession,
  pub callbacks: Vec<ThreadsafeFunction<String>>,
  pub media_props_token: Option<EventRegistrationToken>,
  pub playback_info_token: Option<EventRegistrationToken>,
  pub timeline_props_token: Option<EventRegistrationToken>,
}

pub struct SessionManager {
  pub sessions: HashMap<String, InnerSession>,
  pub session_added_callbacks: Vec<ThreadsafeFunction<MediaInfo>>,
  pub session_removed_callbacks: Vec<ThreadsafeFunction<String>>,
  pub media_props_callbacks: Vec<ThreadsafeFunction<MediaPropsCallbackData>>,
  pub playback_info_callbacks: Vec<ThreadsafeFunction<PlaybackInfoCallbackData>>,
  pub timeline_props_callbacks: Vec<ThreadsafeFunction<TimelinePropsCallbackData>>,
  pub current_session_changed_callbacks: Vec<ThreadsafeFunction<String>>,
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
      current_session_changed_callbacks: Vec::new(),
    }
  }

  pub fn clear_all_sessions(&mut self) {
    for session_data in self.sessions.values_mut() {
      // 移除所有监听器
      Self::remove_event_token(
        &session_data.session,
        session_data.media_props_token.take(),
        |session, token| session.RemoveMediaPropertiesChanged(token),
      );

      Self::remove_event_token(
        &session_data.session,
        session_data.playback_info_token.take(),
        |session, token| session.RemovePlaybackInfoChanged(token),
      );

      Self::remove_event_token(
        &session_data.session,
        session_data.timeline_props_token.take(),
        |session, token| session.RemoveTimelinePropertiesChanged(token),
      );
    }

    self.sessions.clear();
  }

  // 辅助方法，用于移除之前监听器注册出的令牌
  fn remove_event_token<F>(
    session: &GlobalSystemMediaTransportControlsSession,
    token: Option<EventRegistrationToken>,
    remove_fn: F,
  ) where
    F: FnOnce(
      &GlobalSystemMediaTransportControlsSession,
      EventRegistrationToken,
    ) -> windows::core::Result<()>,
  {
    if let Some(token) = token {
      let _ = remove_fn(session, token);
    }
  }
}

pub fn register_session(
  inner: &mut SessionManager,
  id: String,
  session: GlobalSystemMediaTransportControlsSession,
) {
  // 媒体属性变化
  let media_props_token =
    register_media_props_handler(&session, &inner.media_props_callbacks, id.clone());

  // 播放信息变化
  let playback_token =
    register_playback_info_handler(&session, &inner.playback_info_callbacks, id.clone());

  // 时间线变化
  let timeline_token =
    register_timeline_props_handler(&session, &inner.timeline_props_callbacks, id.clone());

  inner.sessions.insert(
    id.clone(),
    InnerSession {
      session: session.clone(),
      callbacks: Vec::new(),
      media_props_token,
      playback_info_token: playback_token,
      timeline_props_token: timeline_token,
    },
  );

  if let Ok(Some(media_info)) = utils::get_media_info_for_session(&session) {
    for callback in &inner.session_added_callbacks {
      callback.call(Ok(media_info.clone()), ThreadsafeFunctionCallMode::Blocking);
    }
  }
}

fn register_media_props_handler(
  session: &GlobalSystemMediaTransportControlsSession,
  callbacks: &[ThreadsafeFunction<MediaPropsCallbackData>],
  id: String,
) -> Option<EventRegistrationToken> {
  let media_session_clone = session.clone();
  let media_props_callbacks = callbacks.to_vec();

  session
    .MediaPropertiesChanged(&TypedEventHandler::new(move |_, _| {
      if let Ok(Some(media_props)) = utils::get_media_props_for_session(&media_session_clone) {
        for callback in &media_props_callbacks {
          callback.call(
            Ok(MediaPropsCallbackData {
              source_app_id: id.clone(),
              media_props: media_props.clone(),
            }),
            ThreadsafeFunctionCallMode::Blocking,
          );
        }
      }
      Ok(())
    }))
    .ok()
}

fn register_playback_info_handler(
  session: &GlobalSystemMediaTransportControlsSession,
  callbacks: &[ThreadsafeFunction<PlaybackInfoCallbackData>],
  id: String,
) -> Option<EventRegistrationToken> {
  let playback_session_clone = session.clone();
  let playback_info_callbacks = callbacks.to_vec();

  session
    .PlaybackInfoChanged(&TypedEventHandler::new(move |_, _| {
      if let Ok(Some(playback_info)) = utils::get_playback_info_for_session(&playback_session_clone)
      {
        for callback in &playback_info_callbacks {
          callback.call(
            Ok(PlaybackInfoCallbackData {
              source_app_id: id.clone(),
              playback_info: playback_info.clone(),
            }),
            ThreadsafeFunctionCallMode::Blocking,
          );
        }
      }
      Ok(())
    }))
    .ok()
}

fn register_timeline_props_handler(
  session: &GlobalSystemMediaTransportControlsSession,
  callbacks: &[ThreadsafeFunction<TimelinePropsCallbackData>],
  id: String,
) -> Option<EventRegistrationToken> {
  let timeline_session_clone = session.clone();
  let timeline_props_callbacks = callbacks.to_vec();

  session
    .TimelinePropertiesChanged(&TypedEventHandler::new(move |_, _| {
      if let Ok(Some(timeline_props)) =
        utils::get_timeline_props_for_session(&timeline_session_clone)
      {
        for callback in &timeline_props_callbacks {
          callback.call(
            Ok(TimelinePropsCallbackData {
              source_app_id: id.clone(),
              timeline_props: timeline_props.clone(),
            }),
            ThreadsafeFunctionCallMode::Blocking,
          );
        }
      }
      Ok(())
    }))
    .ok()
}
