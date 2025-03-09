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
  let sessions = match manager.GetSessions() {
    Ok(s) => s,
    Err(_) => return Ok(Vec::new()),
  };
  
  let size = match sessions.Size() {
    Ok(s) => s,
    Err(_) => return Ok(Vec::new()),
  };

  let mut result = Vec::new();
  for i in 0..size {
    let session = match sessions.GetAt(i) {
      Ok(s) => s,
      Err(_) => continue,
    };
    
    if let Ok(Some(info)) = utils::get_media_info_for_session(&session) {
      result.push(info);
    }
  }

  Ok(result)
}

#[napi]
pub fn get_session_by_id(source_app_id: String) -> Result<Option<MediaInfo>> {
  let manager = create_manager()?;
  let sessions = match manager.GetSessions() {
    Ok(s) => s,
    Err(_) => return Ok(None),
  };
  
  let size = match sessions.Size() {
    Ok(s) => s, 
    Err(_) => return Ok(None),
  };

  for i in 0..size {
    let session = match sessions.GetAt(i) {
      Ok(s) => s,
      Err(_) => continue,
    };
    
    let id = match session.SourceAppUserModelId() {
      Ok(id) => id,
      Err(_) => continue,
    };
    
    if id.to_string() == source_app_id {
      return utils::get_media_info_for_session(&session);
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