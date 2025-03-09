/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export declare function getCurrentSession(): MediaInfo | null
export declare function getSessions(): Array<MediaInfo>
export declare function getSessionById(sourceAppId: string): MediaInfo | null
export interface MediaPropsCallbackData {
  sourceAppId: string
  mediaProps: MediaProps
}
export interface PlaybackInfoCallbackData {
  sourceAppId: string
  playbackInfo: PlaybackInfo
}
export interface TimelinePropsCallbackData {
  sourceAppId: string
  timelineProps: TimelineProps
}
export interface TimelineProps {
  position: number
  duration: number
}
export interface PlaybackInfo {
  playbackStatus: number
  playbackType: number
}
export interface MediaProps {
  title: string
  artist: string
  albumTitle: string
  albumArtist: string
  genres: Array<string>
  albumTrackCount: number
  trackNumber: number
  thumbnail?: Buffer | undefined
}
export interface MediaInfo {
  sourceAppId: string
  media: MediaProps
  playback: PlaybackInfo
  timeline: TimelineProps
  lastUpdatedTime: number
}
export declare class SMTCMonitor {
  constructor()
  initialize(): void
  onSessionAdded(callback: (error:unknown, media: MediaInfo) => void): void
  onSessionRemoved(callback: (error:unknown, sourceAppId: string) => void): void
  onMediaPropertiesChanged(callback: (error:unknown, data: {sourceAppId: string, mediaProps: MediaProps}) => void): void
  onPlaybackInfoChanged(callback: (error:unknown, data: {sourceAppId: string, playbackInfo: PlaybackInfo}) => void): void
  onTimelinePropertiesChanged(callback: (error:unknown, data: {sourceAppId: string, timelineProps: TimelineProps}) => void): void
  onCurrentSessionChanged(callback: (error:unknown, sourceAppId: string) => void): void
  destroy(): void
}
