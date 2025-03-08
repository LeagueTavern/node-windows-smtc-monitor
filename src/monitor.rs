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

use crate::media_control;
use crate::session_manager::{self, SessionManager};
use crate::types::MediaInfo;
use crate::utils::win_to_napi_err;

#[napi(js_name = "SMTCMonitor")]
pub struct SMTCMonitor {
  manager: Arc<Mutex<SessionManager>>,
  smtc_manager: Option<GlobalSystemMediaTransportControlsSessionManager>,
  sessions_changed_token: Option<EventRegistrationToken>,
}

#[napi]
impl SMTCMonitor {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      manager: Arc::new(Mutex::new(SessionManager::new())),
      smtc_manager: None,
      sessions_changed_token: None,
    }
  }

  #[napi]
  pub fn initialize(&mut self) -> Result<()> {
    self.smtc_manager = match media_control::create_manager() {
      Ok(manager) => Some(manager),
      Err(e) => return Err(e),
    };

    let manager = self.smtc_manager.as_ref().unwrap().clone();
    let manager_clone = manager.clone();
    let inner_manager = self.manager.clone();

    self.scan_existing_sessions()?;

    // 监听会话变更事件，在回调内避免使用 NAPI 错误处理
    let token = win_to_napi_err(
      manager.SessionsChanged(&TypedEventHandler::new(move |_, _| {
        let manager = manager_clone.clone();
        if let Ok(sessions) = manager.GetSessions() {
          let mut inner = inner_manager.lock().unwrap();

          let mut current_ids = Vec::new();

          if let Ok(size) = sessions.Size() {
            for i in 0..size {
              if let Ok(session) = sessions.GetAt(i) {
                if let Ok(id) = session.SourceAppUserModelId() {
                  let id = id.to_string();
                  current_ids.push(id.clone());

                  if !inner.sessions.contains_key(&id) {
                    Self::register_session(&mut inner, id.clone(), session);
                  }
                }
              }
            }

            let mut removed_ids = Vec::new();
            for id in inner.sessions.keys() {
              if !current_ids.contains(id) {
                removed_ids.push(id.clone());
              }
            }

            for id in removed_ids {
              inner.sessions.remove(&id);
              // 通知会话已移除
              for callback in &inner.session_removed_callbacks {
                callback.call(Ok(id.clone()), ThreadsafeFunctionCallMode::Blocking);
              }
            }
          }
        }
        Ok(())
      })),
    )?;

    // 存储令牌以便之后可以取消注册
    self.sessions_changed_token = Some(token);

    Ok(())
  }

  #[napi(ts_args_type = "callback: (error:unknown, media: MediaInfo) => void")]
  pub fn on_session_added(&mut self, callback: JsFunction) -> Result<()> {
    let tsfn: ThreadsafeFunction<MediaInfo> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let mut inner = self.manager.lock().unwrap();
    inner.session_added_callbacks.push(tsfn);
    Ok(())
  }

  #[napi(ts_args_type = "callback: (error:unknown, sourceAppId: MediaInfo) => void")]
  pub fn on_session_removed(&mut self, callback: JsFunction) -> Result<()> {
    let tsfn: ThreadsafeFunction<String> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let mut inner = self.manager.lock().unwrap();
    inner.session_removed_callbacks.push(tsfn);
    Ok(())
  }

  #[napi(ts_args_type = "callback: (error:unknown, media: MediaInfo) => void")]
  pub fn on_media_properties_changed(&mut self, callback: JsFunction) -> Result<()> {
    let tsfn: ThreadsafeFunction<MediaInfo> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let mut inner = self.manager.lock().unwrap();
    inner.media_props_callbacks.push(tsfn);
    Ok(())
  }

  #[napi(ts_args_type = "callback: (error:unknown, media: MediaInfo) => void")]
  pub fn on_playback_info_changed(&mut self, callback: JsFunction) -> Result<()> {
    let tsfn: ThreadsafeFunction<MediaInfo> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let mut inner = self.manager.lock().unwrap();
    inner.playback_info_callbacks.push(tsfn);
    Ok(())
  }

  #[napi(ts_args_type = "callback: (error:unknown, media: MediaInfo) => void")]
  pub fn on_timeline_properties_changed(&mut self, callback: JsFunction) -> Result<()> {
    let tsfn: ThreadsafeFunction<MediaInfo> =
      callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let mut inner = self.manager.lock().unwrap();
    inner.timeline_props_callbacks.push(tsfn);
    Ok(())
  }

  #[napi]
  pub fn destroy(&mut self) -> Result<()> {
    if let (Some(manager), Some(token)) = (&self.smtc_manager, self.sessions_changed_token.take()) {
      if let Err(e) = manager.RemoveSessionsChanged(token) {
        eprintln!("Failed to remove sessions changed event handler: {}", e);
      }
    }

    match self.manager.lock() {
      Ok(mut inner) => {
        inner.clear_all_sessions();

        inner.session_added_callbacks.clear();
        inner.session_removed_callbacks.clear();
        inner.media_props_callbacks.clear();
        inner.playback_info_callbacks.clear();
        inner.timeline_props_callbacks.clear();
      }
      Err(e) => {
        eprintln!("Failed to acquire lock on session manager: {}", e);
      }
    }

    self.smtc_manager = None;
    Ok(())
  }

  fn get_manager(&self) -> Result<GlobalSystemMediaTransportControlsSessionManager> {
    if let Some(manager) = &self.smtc_manager {
      return Ok(manager.clone());
    }

    Err(Error::new(
      Status::GenericFailure,
      "SMTCMonitor not initialized. Please call initialize() first.".to_string(),
    ))
  }

  fn scan_existing_sessions(&mut self) -> Result<()> {
    let manager = self.get_manager()?;
    if let Ok(sessions) = manager.GetSessions() {
      let mut inner = self.manager.lock().unwrap();

      if let Ok(size) = sessions.Size() {
        for i in 0..size {
          if let Ok(session) = sessions.GetAt(i) {
            if let Ok(id) = session.SourceAppUserModelId() {
              let id = id.to_string();

              // 如果是新会话，添加到管理器并设置监听
              if !inner.sessions.contains_key(&id) {
                Self::register_session(&mut inner, id.clone(), session);
              }
            }
          }
        }
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
