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
    // 使用 ? 运算符简化错误处理
    self.smtc_manager = Some(media_control::create_manager()?);

    let manager = self.smtc_manager.as_ref().unwrap().clone();
    let manager_clone = manager.clone();
    let inner_manager = self.manager.clone();

    self.scan_existing_sessions()?;

    // 使用函数抽象简化事件处理逻辑
    let token = win_to_napi_err(
      manager.SessionsChanged(&TypedEventHandler::new(move |_, _| {
        Self::handle_sessions_changed(&manager_clone, &inner_manager);
        Ok(())
      })),
    )?;

    // 存储令牌以便之后可以取消注册
    self.sessions_changed_token = Some(token);

    Ok(())
  }

  // 将会话变更处理逻辑抽象为单独函数
  fn handle_sessions_changed(
    manager: &GlobalSystemMediaTransportControlsSessionManager,
    inner_manager: &Arc<Mutex<SessionManager>>,
  ) {
    if let Ok(sessions) = manager.GetSessions() {
      if let Ok(mut inner) = inner_manager.lock() {
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

          // 使用迭代器方法找出已删除的会话
          let removed_ids: Vec<_> = inner
            .sessions
            .keys()
            .filter(|id| !current_ids.contains(id))
            .cloned()
            .collect();

          for id in removed_ids {
            inner.sessions.remove(&id);
            // 通知会话已移除
            for callback in &inner.session_removed_callbacks {
              callback.call(Ok(id.clone()), ThreadsafeFunctionCallMode::Blocking);
            }
          }
        }
      }
    }
  }

  // 以下是事件处理函数
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
    // 解绑事件监听器
    if let (Some(manager), Some(token)) = (&self.smtc_manager, self.sessions_changed_token.take()) {
      let _ = manager.RemoveSessionsChanged(token);
    }

    // 清理资源
    if let Ok(mut inner) = self.manager.lock() {
      inner.clear_all_sessions();

      inner.session_added_callbacks.clear();
      inner.session_removed_callbacks.clear();
      inner.media_props_callbacks.clear();
      inner.playback_info_callbacks.clear();
      inner.timeline_props_callbacks.clear();
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

    if let Ok(sessions) = manager.GetSessions() {
      if let Ok(mut inner) = self.manager.lock() {
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
