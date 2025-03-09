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

use crate::{types::MediaInfo, MediaProps, PlaybackInfo, TimelineProps};

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

// 安全地运行Windows API调用并处理可能的错误
pub fn try_win_api<T, F>(op: F) -> Option<T>
where
  F: FnOnce() -> core::Result<T>,
{
  op().ok()
}

pub fn get_media_props_for_session(
  session: &GlobalSystemMediaTransportControlsSession,
) -> Result<Option<MediaProps>> {
  // 尝试获取媒体属性
  let media_props = match try_win_api(|| {
    session
      .TryGetMediaPropertiesAsync()
      .and_then(|props_async| props_async.get())
  }) {
    Some(props) => props,
    None => return Ok(None),
  };

  // 安全地获取基本属性
  let title = win_to_napi_err(media_props.Title())?.to_string();
  let artist = win_to_napi_err(media_props.Artist())?.to_string();
  let album_title = win_to_napi_err(media_props.AlbumTitle())?.to_string();
  let album_artist = win_to_napi_err(media_props.AlbumArtist())?.to_string();

  // 获取流派列表
  let genres = match media_props.Genres() {
    Ok(genre_list) => {
      let size = win_to_napi_err(genre_list.Size())?;
      let mut result = Vec::with_capacity(size as usize);

      for i in 0..size {
        if let Ok(genre) = genre_list.GetAt(i) {
          result.push(genre.to_string());
        }
      }

      result
    }
    Err(_) => Vec::new(),
  };

  let album_track_count = win_to_napi_err(media_props.AlbumTrackCount())?;
  let track_number = win_to_napi_err(media_props.TrackNumber())?;

  // 提取缩略图
  let thumbnail = try_win_api(|| media_props.Thumbnail())
    .and_then(|thumbnail| try_win_api(|| thumbnail.OpenReadAsync().and_then(|op| op.get())))
    .and_then(|stream| {
      try_win_api(|| WinBuffer::Create(1024 * 1024)).and_then(|buffer| {
        try_win_api(|| buffer.Capacity()).and_then(|capacity| {
          try_win_api(|| {
            stream
              .ReadAsync(&buffer, capacity, InputStreamOptions::None)
              .and_then(|read_op| read_op.get())
          })
          .and_then(|_| buffer_to_napi_buffer(&buffer).ok().flatten())
        })
      })
    });

  Ok(Some(MediaProps {
    title,
    artist,
    album_title,
    album_artist,
    genres,
    album_track_count: album_track_count.try_into().unwrap_or(0),
    track_number: track_number.try_into().unwrap_or(0),
    thumbnail,
  }))
}

pub fn get_playback_info_for_session(
  session: &GlobalSystemMediaTransportControlsSession,
) -> Result<Option<PlaybackInfo>> {
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

  let playback_type = try_win_api(|| playback_info.PlaybackType().and_then(|pt| pt.Value()))
    .map(|pt| match pt {
      MediaPlaybackType::Unknown => 0,
      MediaPlaybackType::Music => 1,
      MediaPlaybackType::Video => 2,
      MediaPlaybackType::Image => 3,
      _ => 0,
    })
    .unwrap_or(0);

  Ok(Some(PlaybackInfo {
    playback_status,
    playback_type,
  }))
}

pub fn get_timeline_props_for_session(
  session: &GlobalSystemMediaTransportControlsSession,
) -> Result<Option<TimelineProps>> {
  let timeline_props = win_to_napi_err(session.GetTimelineProperties())?;
  let position = timespan_to_seconds(win_to_napi_err(timeline_props.Position())?);
  let duration = timespan_to_seconds(win_to_napi_err(timeline_props.EndTime())?);

  Ok(Some(TimelineProps { position, duration }))
}

pub fn get_media_info_for_session(
  session: &GlobalSystemMediaTransportControlsSession,
) -> Result<Option<MediaInfo>> {
  // 获取应用ID
  let source_app_id = match win_to_napi_err(session.SourceAppUserModelId()) {
    Ok(id) => id.to_string(),
    Err(_) => return Ok(None),
  };

  // 使用提取操作符简化代码结构
  let media = get_media_props_for_session(session)?
    .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to get media properties"))?;

  let playback = get_playback_info_for_session(session)?
    .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to get playback info"))?;

  let timeline = get_timeline_props_for_session(session)?
    .ok_or_else(|| Error::new(Status::GenericFailure, "Failed to get timeline properties"))?;

  let last_updated_time = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis() as f64;

  Ok(Some(MediaInfo {
    source_app_id,
    media,
    playback,
    timeline,
    last_updated_time,
  }))
}
