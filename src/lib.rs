#![deny(clippy::all)]

// 🤓☝️
// 找了半天，竟然没有人做这个，那就自己写一个

#[macro_use]
extern crate napi_derive;

mod session_manager;
mod types;
mod utils;

use napi::{
  bindgen_prelude::*,
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  JsFunction, Result,
};
use std::sync::{Arc, Mutex};
use windows::{
  Foundation::TypedEventHandler,
  Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
  },
};

use crate::session_manager::SessionManager;
use crate::types::MediaInfo;
use crate::utils::win_to_napi_err;

#[napi(js_name = "SMTCMonitor")]
pub struct SMTCMonitor {
  manager: Arc<Mutex<SessionManager>>,
  smtc_manager: Option<GlobalSystemMediaTransportControlsSessionManager>,
}

#[napi]
impl SMTCMonitor  {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      manager: Arc::new(Mutex::new(SessionManager::new())),
      smtc_manager: None,
    }
  }

  #[napi]
  pub fn initialize(&mut self) -> Result<()> {
    let manager = self.get_manager()?;
    let manager_clone = manager.clone();
    let inner_manager = self.manager.clone();

    // 初始化时主动扫描并注册现有会话，解决第三个问题
    self.scan_existing_sessions()?;

    // 监听会话变更事件，在回调内避免使用 NAPI 错误处理
    let _token = win_to_napi_err(
      manager.SessionsChanged(&TypedEventHandler::new(move |_, _| {
        let manager = manager_clone.clone();
        if let Ok(sessions) = manager.GetSessions() {
          let mut inner = inner_manager.lock().unwrap();

          // 检测新加入的会话
          let mut current_ids = Vec::new();

          // 使用 windows 标准错误处理，不用 NAPI 的错误处理
          if let Ok(size) = sessions.Size() {
            for i in 0..size {
              if let Ok(session) = sessions.GetAt(i) {
                if let Ok(id) = session.SourceAppUserModelId() {
                  let id = id.to_string();
                  current_ids.push(id.clone());

                  // 如果是新会话，添加到管理器并触发回调
                  if !inner.sessions.contains_key(&id) {
                    // 使用辅助方法注册会话
                    Self::register_session(&mut inner, id.clone(), session);
                  }
                }
              }
            }

            // 检测已移除的会话
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

    Ok(())
  }

  // 修改回调参数类型
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
    // 会话移除事件仍保留原样，只传递 ID
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

  // 修改异步方法为同步方法，避免 Send trait 问题
  #[napi]
  pub fn get_current_session(&self) -> Result<Option<MediaInfo>> {
    let manager = self.get_manager()?;
    if let Ok(session) = manager.GetCurrentSession() {
      return self.session_to_media_info_sync(&session);
    }
    Ok(None)
  }

  #[napi]
  pub fn get_all_sessions(&self) -> Result<Vec<MediaInfo>> {
    let manager = self.get_manager()?;

    // 使用同步方式处理，避免 Send 问题
    let mut result = Vec::new();
    if let Ok(sessions) = manager.GetSessions() {
      if let Ok(size) = sessions.Size() {
        for i in 0..size {
          if let Ok(session) = sessions.GetAt(i) {
            if let Ok(Some(info)) = self.session_to_media_info_sync(&session) {
              result.push(info);
            }
          }
        }
      }
    }

    Ok(result)
  }

  #[napi]
  pub fn get_session_by_id(&self, source_app_id: String) -> Result<Option<MediaInfo>> {
    let manager = self.get_manager()?;

    if let Ok(sessions) = manager.GetSessions() {
      if let Ok(size) = sessions.Size() {
        for i in 0..size {
          if let Ok(session) = sessions.GetAt(i) {
            if let Ok(id) = session.SourceAppUserModelId() {
              if id.to_string() == source_app_id {
                return self.session_to_media_info_sync(&session);
              }
            }
          }
        }
      }
    }

    Ok(None)
  }

  fn get_manager(&self) -> Result<GlobalSystemMediaTransportControlsSessionManager> {
    if let Some(manager) = &self.smtc_manager {
      return Ok(manager.clone());
    }

    match GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
      Ok(operation) => match operation.get() {
        Ok(manager) => Ok(manager),
        Err(e) => Err(Error::new(Status::GenericFailure, e.to_string())),
      },
      Err(e) => Err(Error::new(Status::GenericFailure, e.to_string())),
    }
  }

  fn session_to_media_info_sync(
    &self,
    session: &GlobalSystemMediaTransportControlsSession,
  ) -> Result<Option<MediaInfo>> {
    // 使用公共方法获取 MediaInfo
    Self::get_media_info_for_session(session)
  }

  // 添加扫描现有会话的辅助方法
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
                // 使用辅助方法注册会话
                Self::register_session(&mut inner, id.clone(), session);
              }
            }
          }
        }
      }
    }

    Ok(())
  }

  // 添加用于回调的公共方法，获取会话的 MediaInfo
  fn get_media_info_for_session(
    session: &GlobalSystemMediaTransportControlsSession,
  ) -> Result<Option<MediaInfo>> {
    // 方法实现移至utils::get_media_info_for_session
    utils::get_media_info_for_session(session)
  }

  // 注册会话方法
  fn register_session(
    inner: &mut SessionManager,
    id: String,
    session: GlobalSystemMediaTransportControlsSession,
  ) {
    // 实现移至session_manager::register_session
    session_manager::register_session(inner, id, session);
  }
}
