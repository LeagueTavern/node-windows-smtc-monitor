/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface MediaInfo {
  sourceAppId: string
  title: string
  artist: string
  albumTitle: string
  albumArtist: string
  genres: Array<string>
  albumTrackCount: number
  trackNumber: number
  thumbnail?: Buffer | undefined
  playbackStatus: number
  playbackType: number
  position: number
  duration: number
  lastUpdatedTime: number
}
export declare class SMTCMonitor {
  constructor()
  initialize(): void
  onSessionAdded(callback: (error:unknown, media: MediaInfo) => void): void
  onSessionRemoved(callback: (error:unknown, sourceAppId: MediaInfo) => void): void
  onMediaPropertiesChanged(callback: (error:unknown, media: MediaInfo) => void): void
  onPlaybackInfoChanged(callback: (error:unknown, media: MediaInfo) => void): void
  onTimelinePropertiesChanged(callback: (error:unknown, media: MediaInfo) => void): void
  getCurrentSession(): MediaInfo | null
  getAllSessions(): Array<MediaInfo>
  getSessionById(sourceAppId: string): MediaInfo | null
  destroy(): void
}
