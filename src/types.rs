use std::fmt;
use napi_derive::napi;

#[napi(object)]
#[derive(Clone)]
pub struct MediaInfo {
  pub source_app_id: String,
  pub title: String,
  pub artist: String,
  pub album_title: String,
  pub album_artist: String,
  pub genres: Vec<String>,
  pub album_track_count: u32,
  pub track_number: u32,
  #[napi(ts_type = "Buffer | undefined")]
  pub thumbnail: Option<napi::bindgen_prelude::Buffer>,
  pub playback_status: u8,
  pub playback_type: u8,
  pub position: f64,
  pub duration: f64,
  pub last_updated_time: f64,
}

// 手动实现 Debug trait，忽略 thumbnail 字段
impl fmt::Debug for MediaInfo {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("MediaInfo")
      .field("source_app_id", &self.source_app_id)
      .field("title", &self.title)
      .field("artist", &self.artist)
      .field("album_title", &self.album_title)
      .field("album_artist", &self.album_artist)
      .field("genres", &self.genres)
      .field("album_track_count", &self.album_track_count)
      .field("track_number", &self.track_number)
      .field("thumbnail", &"[Buffer]") // 忽略实际内容，只显示占位符
      .field("playback_status", &self.playback_status)
      .field("playback_type", &self.playback_type)
      .field("position", &self.position)
      .field("duration", &self.duration)
      .field("last_updated_time", &self.last_updated_time)
      .finish()
  }
}
