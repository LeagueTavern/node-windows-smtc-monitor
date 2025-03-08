import { EventEmitter } from "events"
import { MediaInfo, SMTCMonitor as SMTC } from "./binding"

/**
 * Global System Media Transport Controls Session Playback Status
 */
export enum PlaybackStatus {
  CLOSED = 0,
  OPENED = 1,
  CHANGING = 2,
  STOPPED = 3,
  PLAYING = 4,
  PAUSED = 5,
}

declare class SMTCMonitor extends EventEmitter {
  constructor()

  private smtc: SMTC
  private _mediaSessions: Map<string, MediaInfo>

  private _preloadSessions(): void
  private _bindEvents(): void
  private _onMediaPropertiesChanged(media: MediaInfo): void
  private _onTimelinePropertiesChanged(media: MediaInfo): void
  private _onPlaybackInfoChanged(media: MediaInfo): void
  private _onSessionAdded(media: MediaInfo): void
  private _onSessionRemoved(sourceAppId: string): void

  getAllMediaSessions(): MediaInfo[]
  getCurrentMediaSession(): MediaInfo | null
  getMediaSessionByAppId(sourceAppId: string): MediaInfo | null

  on(event: "session-media-changed", listener: (media: MediaInfo) => void): this
  on(
    event: "session-timeline-changed",
    listener: (media: MediaInfo) => void
  ): this
  on(
    event: "session-playback-changed",
    listener: (media: MediaInfo) => void
  ): this
  on(event: "session-added", listener: (media: MediaInfo) => void): this
  on(event: "session-removed", listener: (sourceAppId: string) => void): this
}

export { SMTCMonitor }
