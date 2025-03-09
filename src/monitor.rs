use napi::{
  bindgen_prelude::*,
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  JsFunction, Result,
};
use std::sync::{Arc, Mutex};
use windows::{
  Foundation::{EventRegistrationToken, TypedEventHandler},
  Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
  },
};

use crate::session_manager::{self, SessionManager};
use crate::types::MediaInfo;
use crate::utils::win_to_napi_err;
use crate::{media_control, MediaProps, PlaybackInfo, TimelineProps};

#[napi(object)]
pub struct MediaPropsCallbackData {
  pub source_app_id: String,
  pub media_props: MediaProps,
}

#[napi(object)]
pub struct PlaybackInfoCallbackData {
  pub source_app_id: String,
  pub playback_info: PlaybackInfo,
}

#[napi(object)]
pub struct TimelinePropsCallbackData {
  pub source_app_id: String,
  pub timeline_props: TimelineProps,
}

#[napi(js_name = "SMTCMonitor")]
pub struct SMTCMonitor {
  manager: Arc<Mutex<SessionManager>>,
  smtc_manager: Option<GlobalSystemMediaTransportControlsSessionManager>,
  sessions_changed_token: Option<EventRegistrationToken>,
  current_session_changed_token: Option<EventRegistrationToken>,
}

#[napi]
impl SMTCMonitor {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      manager: Arc::new(Mutex::new(SessionManager::new())),
      smtc_manager: None,
      sessions_changed_token: None,
      current_session_changed_token: None,
    }
  }

  #[napi]
  pub fn initialize(&mut self) -> Result<()> {
    self.smtc_manager = Some(media_control::create_manager()?);

    let manager = self.smtc_manager.as_ref().unwrap().clone();
    let manager_clone = manager.clone();
    let inner_manager = self.manager.clone();

    self.scan_existing_sessions()?;

    let token = win_to_napi_err(
      manager.SessionsChanged(&TypedEventHandler::new(move |_, _| {
        Self::handle_sessions_changed(&manager_clone, &inner_manager);
        Ok(())
      })),
    )?;

    self.sessions_changed_token = Some(token);

    // 监听当前会话变化
    let manager_clone = manager.clone();
    let inner_manager = self.manager.clone();
    let current_session_token = win_to_napi_err(
      manager.CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
        Self::handle_current_session_changed(&manager_clone, &inner_manager);
        Ok(())
      })),
    )?;

    self.current_session_changed_token = Some(current_session_token);

    Ok(())
  }

  fn handle_sessions_changed(
    manager: &GlobalSystemMediaTransportControlsSessionManager,
    inner_manager: &Arc<Mutex<SessionManager>>,
  ) {
    let sessions = match manager.GetSessions() {
      Ok(s) => s,
      Err(_) => return,
    };

    let inner = match inner_manager.lock() {
      Ok(inner) => inner,
      Err(_) => return,
    };

    let mut inner = inner; // 转为可变引用
    let mut current_ids = Vec::new();

    let size = match sessions.Size() {
      Ok(size) => size,
      Err(_) => return,
    };

    // 处理现有会话
    for i in 0..size {
      let session = match sessions.GetAt(i) {
        Ok(s) => s,
        Err(_) => continue,
      };

      let id = match session.SourceAppUserModelId() {
        Ok(id) => id.to_string(),
        Err(_) => continue,
      };

      current_ids.push(id.clone());

      if !inner.sessions.contains_key(&id) {
        Self::register_session(&mut inner, id.clone(), session);
      }
    }

    // 处理已移除的会话
    let removed_ids: Vec<_> = inner
      .sessions
      .keys()
      .filter(|id| !current_ids.contains(id))
      .cloned()
      .collect();

    for id in removed_ids {
      inner.sessions.remove(&id);
      for callback in &inner.session_removed_callbacks {
        callback.call(Ok(id.clone()), ThreadsafeFunctionCallMode::Blocking);
      }
    }
  }

  fn handle_current_session_changed(
    manager: &GlobalSystemMediaTransportControlsSessionManager,
    inner_manager: &Arc<Mutex<SessionManager>>,
  ) {
    let current_session = match manager.GetCurrentSession() {
      Ok(session) => session,
      Err(_) => return,
    };

    let source_app_id = match current_session.SourceAppUserModelId() {
      Ok(id) => id.to_string(),
      Err(_) => return,
    };

    if let Ok(inner) = inner_manager.lock() {
      for callback in &inner.current_session_changed_callbacks {
        callback.call(Ok(source_app_id.clone()), ThreadsafeFunctionCallMode::Blocking);
      }
    }
  }

  #[napi(ts_args_type = "callback: (error:unknown, media: MediaInfo) => void")]
  pub fn on_session_added(&mut self, callback: JsFunction) -> Result<()> {
    let tsfn: ThreadsafeFunction<MediaInfo> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let mut inner = self.manager.lock().unwrap();
    inner.session_added_callbacks.push(tsfn);
    Ok(())
  }

  #[napi(ts_args_type = "callback: (error:unknown, sourceAppId: string) => void")]
  pub fn on_session_removed(&mut self, callback: JsFunction) -> Result<()> {
    let tsfn: ThreadsafeFunction<String> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let mut inner = self.manager.lock().unwrap();
    inner.session_removed_callbacks.push(tsfn);
    Ok(())
  }

  #[napi(
    ts_args_type = "callback: (error:unknown, data: {sourceAppId: string, mediaProps: MediaProps}) => void"
  )]
  pub fn on_media_properties_changed(&mut self, callback: JsFunction) -> Result<()> {
    let tsfn: ThreadsafeFunction<MediaPropsCallbackData> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let mut inner = self.manager.lock().unwrap();
    inner.media_props_callbacks.push(tsfn);
    Ok(())
  }

  #[napi(
    ts_args_type = "callback: (error:unknown, data: {sourceAppId: string, playbackInfo: PlaybackInfo}) => void"
  )]
  pub fn on_playback_info_changed(&mut self, callback: JsFunction) -> Result<()> {
    let tsfn: ThreadsafeFunction<PlaybackInfoCallbackData> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let mut inner = self.manager.lock().unwrap();
    inner.playback_info_callbacks.push(tsfn);
    Ok(())
  }

  #[napi(
    ts_args_type = "callback: (error:unknown, data: {sourceAppId: string, timelineProps: TimelineProps}) => void"
  )]
  pub fn on_timeline_properties_changed(&mut self, callback: JsFunction) -> Result<()> {
    let tsfn: ThreadsafeFunction<TimelinePropsCallbackData> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let mut inner = self.manager.lock().unwrap();
    inner.timeline_props_callbacks.push(tsfn);
    Ok(())
  }

  #[napi(ts_args_type = "callback: (error:unknown, sourceAppId: string) => void")]
  pub fn on_current_session_changed(&mut self, callback: JsFunction) -> Result<()> {
    let tsfn: ThreadsafeFunction<String> = 
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let mut inner = self.manager.lock().unwrap();
    inner.current_session_changed_callbacks.push(tsfn);
    Ok(())
  }

  #[napi]
  pub fn destroy(&mut self) -> Result<()> {
    if let (Some(manager), Some(token)) = (&self.smtc_manager, self.sessions_changed_token.take()) {
      let _ = manager.RemoveSessionsChanged(token);
    }

    if let (Some(manager), Some(token)) = (&self.smtc_manager, self.current_session_changed_token.take()) {
      let _ = manager.RemoveCurrentSessionChanged(token);
    }

    if let Ok(mut inner) = self.manager.lock() {
      inner.clear_all_sessions();

      inner.session_added_callbacks.clear();
      inner.session_removed_callbacks.clear();
      inner.media_props_callbacks.clear();
      inner.playback_info_callbacks.clear();
      inner.timeline_props_callbacks.clear();
      inner.current_session_changed_callbacks.clear();
    }

    self.smtc_manager = None;
    Ok(())
  }

  fn get_manager(&self) -> Result<GlobalSystemMediaTransportControlsSessionManager> {
    self.smtc_manager.clone().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "SMTCMonitor not initialized. Please call initialize() first.".to_string(),
      )
    })
  }

  fn scan_existing_sessions(&mut self) -> Result<()> {
    let manager = self.get_manager()?;
    let sessions = match manager.GetSessions() {
      Ok(s) => s,
      Err(_) => return Ok(()),
    };

    let mut inner = match self.manager.lock() {
      Ok(inner) => inner,
      Err(_) => return Ok(()),
    };

    let size = match sessions.Size() {
      Ok(size) => size,
      Err(_) => return Ok(()),
    };

    for i in 0..size {
      let session = match sessions.GetAt(i) {
        Ok(s) => s,
        Err(_) => continue,
      };

      let id = match session.SourceAppUserModelId() {
        Ok(id) => id.to_string(),
        Err(_) => continue,
      };

      if !inner.sessions.contains_key(&id) {
        Self::register_session(&mut inner, id.clone(), session);
      }
    }

    Ok(())
  }

  fn register_session(
    inner: &mut SessionManager,
    id: String,
    session: GlobalSystemMediaTransportControlsSession,
  ) {
    session_manager::register_session(inner, id, session);
  }
}
