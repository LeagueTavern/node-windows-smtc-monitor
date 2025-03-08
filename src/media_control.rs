use napi::{bindgen_prelude::*, Result};
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

use crate::types::MediaInfo;
use crate::utils;

#[napi]
pub fn get_current_session() -> Result<Option<MediaInfo>> {
  let manager = create_manager()?;

  manager
    .GetCurrentSession()
    .ok()
    .map_or(Ok(None), |session| {
      utils::get_media_info_for_session(&session)
    })
}

#[napi]
pub fn get_all_sessions() -> Result<Vec<MediaInfo>> {
  let manager = create_manager()?;

  let mut result = Vec::new();
  if let Ok(sessions) = manager.GetSessions() {
    if let Ok(size) = sessions.Size() {
      for i in 0..size {
        if let Ok(session) = sessions.GetAt(i) {
          if let Ok(Some(info)) = utils::get_media_info_for_session(&session) {
            result.push(info);
          }
        }
      }
    }
  }

  Ok(result)
}

#[napi]
pub fn get_session_by_id(source_app_id: String) -> Result<Option<MediaInfo>> {
  let manager = create_manager()?;

  if let Ok(sessions) = manager.GetSessions() {
    if let Ok(size) = sessions.Size() {
      for i in 0..size {
        if let Ok(session) = sessions.GetAt(i) {
          if let Ok(id) = session.SourceAppUserModelId() {
            if id.to_string() == source_app_id {
              return utils::get_media_info_for_session(&session);
            }
          }
        }
      }
    }
  }

  Ok(None)
}

pub fn create_manager() -> Result<GlobalSystemMediaTransportControlsSessionManager> {
  let operation = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
    .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

  operation
    .get()
    .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
}
