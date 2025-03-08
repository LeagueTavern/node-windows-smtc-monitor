use napi::{bindgen_prelude::Buffer, Error, Result, Status};
use std::time::{SystemTime, UNIX_EPOCH};
use windows::{
  core,
  Foundation::TimeSpan,
  Media::{
    Control::{
      GlobalSystemMediaTransportControlsSession,
      GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    MediaPlaybackType,
  },
  Storage::Streams::{Buffer as WinBuffer, DataReader, InputStreamOptions},
};

use crate::types::MediaInfo;

pub fn win_to_napi_err<T>(result: core::Result<T>) -> Result<T> {
  result.map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
}

pub fn timespan_to_seconds(ts: TimeSpan) -> f64 {
  ts.Duration as f64 / 10_000_000.0
}

pub fn buffer_to_napi_buffer(win_buffer: &WinBuffer) -> Result<Option<Buffer>> {
  let length = win_to_napi_err(win_buffer.Length())?;
  if length > 0 {
    let mut bytes = vec![0u8; length as usize];
    let data_reader = win_to_napi_err(DataReader::FromBuffer(win_buffer))?;
    win_to_napi_err(data_reader.ReadBytes(&mut bytes))?;

    Ok(Some(bytes.into()))
  } else {
    Ok(None)
  }
}

pub fn get_media_info_for_session(
  session: &GlobalSystemMediaTransportControlsSession,
) -> Result<Option<MediaInfo>> {
  if let Ok(media_props) = session.TryGetMediaPropertiesAsync() {
    let media_props = win_to_napi_err(media_props.get())?;

    let title = win_to_napi_err(media_props.Title())?.to_string();
    let artist = win_to_napi_err(media_props.Artist())?.to_string();
    let album_title = win_to_napi_err(media_props.AlbumTitle())?.to_string();
    let album_artist = win_to_napi_err(media_props.AlbumArtist())?.to_string();

    let mut genres = Vec::new();
    if let Ok(genre_list) = media_props.Genres() {
      for i in 0..win_to_napi_err(genre_list.Size())? {
        if let Ok(genre) = genre_list.GetAt(i) {
          genres.push(genre.to_string());
        }
      }
    }

    let album_track_count = win_to_napi_err(media_props.AlbumTrackCount())?;
    let track_number = win_to_napi_err(media_props.TrackNumber())?;

    // 缩略图读取逻辑
    let thumbnail = if let Ok(thumbnail_ref) = media_props.Thumbnail() {
      if let Ok(stream_op) = thumbnail_ref.OpenReadAsync() {
        if let Ok(stream) = win_to_napi_err(stream_op.get()) {
          // 创建合适的 Buffer 来接收数据
          let buffer = win_to_napi_err(WinBuffer::Create(1024 * 1024))?;

          // 需要传递引用
          if let Ok(read_op) = stream.ReadAsync(
            &buffer,
            win_to_napi_err(buffer.Capacity())?,
            InputStreamOptions::None,
          ) {
            if win_to_napi_err(read_op.get()).is_ok() {
              buffer_to_napi_buffer(&buffer)?
            } else {
              None
            }
          } else {
            None
          }
        } else {
          None
        }
      } else {
        None
      }
    } else {
      None
    };

    let playback_info = win_to_napi_err(session.GetPlaybackInfo())?;

    let playback_status = match win_to_napi_err(playback_info.PlaybackStatus())? {
      GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed => 0,
      GlobalSystemMediaTransportControlsSessionPlaybackStatus::Opened => 1,
      GlobalSystemMediaTransportControlsSessionPlaybackStatus::Changing => 2,
      GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped => 3,
      GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => 4,
      GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => 5,
      _ => 0,
    };

    let playback_type = match playback_info.PlaybackType() {
      Ok(pt_ref) => {
        if let Ok(pt) = pt_ref.Value() {
          match pt {
            MediaPlaybackType::Unknown => 0,
            MediaPlaybackType::Music => 1,
            MediaPlaybackType::Video => 2,
            MediaPlaybackType::Image => 3,
            _ => 0,
          }
        } else {
          0
        }
      }
      Err(_) => 0,
    };

    let timeline_props = win_to_napi_err(session.GetTimelineProperties())?;
    let position = timespan_to_seconds(win_to_napi_err(timeline_props.Position())?);
    let duration = timespan_to_seconds(win_to_napi_err(timeline_props.EndTime())?);
    let last_updated_time = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or_default()
      .as_millis() as f64;

    let source_app_id = win_to_napi_err(session.SourceAppUserModelId())?.to_string();

    Ok(Some(MediaInfo {
      source_app_id,
      title,
      artist,
      album_title,
      album_artist,
      genres,
      album_track_count: album_track_count.try_into().unwrap_or(0),
      track_number: track_number.try_into().unwrap_or(0),
      thumbnail,
      playback_status,
      playback_type,
      position,
      duration,
      last_updated_time,
    }))
  } else {
    Ok(None)
  }
}
