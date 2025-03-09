const os = require("os")
const { EventEmitter } = require("events")
const { PlaybackStatus } = require("./constant")
const {
  SMTCMonitor: SMTC,
  getCurrentSession,
  getSessions,
  getSessionById,
} = require("./binding")

class SMTCMonitor extends EventEmitter {
  constructor() {
    super()
    this.smtc = new SMTC()
    this._mediaSessions = new Map()
    this._bindEvents()
    this._initialize()
    this._preloadSessions()
  }

  _initialize() {
    this.smtc.initialize()
  }
  _preloadSessions() {
    SMTCMonitor.getMediaSessions().forEach((session) => {
      this._mediaSessions.set(session.sourceAppId, session)
    })
  }

  _bindEvents() {
    this.smtc.onMediaPropertiesChanged((error, data) => {
      !error && this._onMediaPropertiesChanged(data)
    })

    this.smtc.onTimelinePropertiesChanged((error, data) => {
      !error && this._onTimelinePropertiesChanged(data)
    })

    this.smtc.onPlaybackInfoChanged((error, data) => {
      !error && this._onPlaybackInfoChanged(data)
    })

    this.smtc.onSessionAdded((error, data) => {
      !error && this._onSessionAdded(data)
    })

    this.smtc.onSessionRemoved((error, sourceAppId) => {
      !error && this._onSessionRemoved(sourceAppId)
    })

    this.smtc.onCurrentSessionChanged((error, sourceAppId) => {
      !error && this._onCurrentSessionChanged(sourceAppId)
    })
  }

  _onMediaPropertiesChanged(data) {
    const { sourceAppId, mediaProps } = data
    const session = this._mediaSessions.get(sourceAppId)
    if (!session) {
      return
    }

    session.media = mediaProps
    this.emit("session-media-changed", sourceAppId, mediaProps)
  }

  _onTimelinePropertiesChanged(data) {
    const { sourceAppId, timelineProps } = data
    const session = this._mediaSessions.get(sourceAppId)
    if (!session) {
      return
    }

    session.timeline = timelineProps
    this.emit("session-timeline-changed", sourceAppId, timelineProps)
  }

  _onPlaybackInfoChanged(data) {
    const { sourceAppId, playbackInfo } = data
    const session = this._mediaSessions.get(sourceAppId)
    if (!session) {
      return
    }

    session.playback = playbackInfo
    this.emit("session-playback-changed", sourceAppId, playbackInfo)
  }

  _onSessionAdded(data) {
    const { sourceAppId } = data
    this._mediaSessions.set(sourceAppId, data)
    this.emit("session-added", sourceAppId, data)
  }

  _onSessionRemoved(sourceAppId) {
    if (this._mediaSessions.has(sourceAppId)) {
      this._mediaSessions.delete(sourceAppId)
      this.emit("session-removed", sourceAppId)
    }
  }

  _onCurrentSessionChanged(sourceAppId) {
    if (this._mediaSessions.has(sourceAppId)) {
      this.emit("current-session-changed", sourceAppId)
    }
  }

  get sessions() {
    return Array.from(this._mediaSessions.values())
  }

  static getMediaSessions() {
    return getSessions()
  }

  static getCurrentMediaSession() {
    return getCurrentSession()
  }

  static getMediaSessionByAppId(sourceAppId) {
    return getSessionById(sourceAppId)
  }

  destroy() {
    try {
      this.removeAllListeners()

      if (this.smtc) {
        this.smtc.destroy()
        this.smtc = null
      }

      if (this._mediaSessions) {
        this._mediaSessions.clear()
        this._mediaSessions = null
      }
    } catch (e) {
      console.error("Error during SMTCMonitor destroy:", e)
    }
  }
}

function _checkCompatibility() {
  const version = os.release()
  const globalWarning = `SMTCMonitor is designed to work with Windows.Media.Control namespace, which requires GlobalSystemMediaTransportControlsSessionManager feature.`
  let warnings = []

  if (
    process.platform !== "win32" ||
    !["ia32", "x64", "arm64"].includes(process.arch)
  ) {
    warnings.push(
      `SMTC Feature is not supported on this platform. Please use Windows 10 or later with x64 / ia32 / arm64 architecture.`
    )
  } else if (!version || _compareVersions(version, "10.0.17763") < 0) {
    warnings.push(`Please use Windows 10 version 1809 (10.0.17763) or later.`)
  }

  if (warnings.length > 0) {
    warnings.push(globalWarning)
    warnings.forEach((warning) => console.warn(warning))
  }
}

function _compareVersions(version1, version2) {
  if (!version1) return -1

  const parts1 = version1.split(".")
  const parts2 = version2.split(".")
  const len = Math.max(parts1.length, parts2.length)

  for (let i = 0; i < len; i++) {
    const num1 = parseInt(parts1[i] || 0, 10)
    const num2 = parseInt(parts2[i] || 0, 10)

    if (num1 > num2) return 1
    if (num1 < num2) return -1
  }

  return 0
}

_checkCompatibility()

module.exports = {
  SMTCMonitor,
  PlaybackStatus,
}
