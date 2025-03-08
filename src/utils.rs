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

// 更简洁的错误处理函数
pub fn win_to_napi_err<T>(result: core::Result<T>) -> Result<T> {
  result.map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
}

pub fn timespan_to_seconds(ts: TimeSpan) -> f64 {
  ts.Duration as f64 / 10_000_000.0
}

pub fn buffer_to_napi_buffer(win_buffer: &WinBuffer) -> Result<Option<Buffer>> {
  let length = win_to_napi_err(win_buffer.Length())?;
  if length == 0 {
    return Ok(None);
  }

  let mut bytes = vec![0u8; length as usize];
  let data_reader = win_to_napi_err(DataReader::FromBuffer(win_buffer))?;
  win_to_napi_err(data_reader.ReadBytes(&mut bytes))?;
  
  Ok(Some(bytes.into()))
}

pub fn get_media_info_for_session(
  session: &GlobalSystemMediaTransportControlsSession,
) -> Result<Option<MediaInfo>> {
  let media_props = match session.TryGetMediaPropertiesAsync() {
    Ok(props_async) => win_to_napi_err(props_async.get())?,
    Err(_) => return Ok(None),
  };

  let title = win_to_napi_err(media_props.Title())?.to_string();
  let artist = win_to_napi_err(media_props.Artist())?.to_string();
  let album_title = win_to_napi_err(media_props.AlbumTitle())?.to_string();
  let album_artist = win_to_napi_err(media_props.AlbumArtist())?.to_string();

  let genres = match media_props.Genres() {
    Ok(genre_list) => {
      let size = win_to_napi_err(genre_list.Size())?;
      (0..size)
        .filter_map(|i| genre_list.GetAt(i).ok())
        .map(|g| g.to_string())
        .collect()
    },
    Err(_) => Vec::new(),
  };

  let album_track_count = win_to_napi_err(media_props.AlbumTrackCount())?;
  let track_number = win_to_napi_err(media_props.TrackNumber())?;

  let thumbnail = media_props
    .Thumbnail()
    .ok()
    .and_then(|thumbnail_ref| thumbnail_ref.OpenReadAsync().ok())
    .and_then(|stream_op| win_to_napi_err(stream_op.get()).ok())
    .and_then(|stream| {
      WinBuffer::Create(1024 * 1024)
        .ok()
        .and_then(|buffer| {
          buffer.Capacity().ok().and_then(|capacity| {
            stream
              .ReadAsync(&buffer, capacity, InputStreamOptions::None)
              .ok()
              .and_then(|read_op| {
                if win_to_napi_err(read_op.get()).is_ok() {
                  buffer_to_napi_buffer(&buffer).ok().flatten()
                } else {
                  None
                }
              })
          })
        })
    });

  let playback_info = win_to_napi_err(session.GetPlaybackInfo())?;

  // 使用更简洁的模式匹配
  let playback_status = match win_to_napi_err(playback_info.PlaybackStatus())? {
    GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed => 0,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus::Opened => 1,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus::Changing => 2,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped => 3,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => 4,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => 5,
    _ => 0,
  };

  let playback_type = playback_info.PlaybackType()
    .ok()
    .and_then(|pt_ref| pt_ref.Value().ok())
    .map(|pt| match pt {
      MediaPlaybackType::Unknown => 0,
      MediaPlaybackType::Music => 1,
      MediaPlaybackType::Video => 2,
      MediaPlaybackType::Image => 3,
      _ => 0,
    })
    .unwrap_or(0);

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
}
