const os = require("os")
const { EventEmitter } = require("events")
const { PlaybackStatus } = require("./constant")
const {
  SMTCMonitor: SMTC,
  getCurrentSession,
  getAllSessions,
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
    SMTCMonitor.getCurrentMediaSessions().forEach((session) => {
      this._mediaSessions.set(session.sourceAppId, session)
    })
  }

  _bindEvents() {
    this.smtc.onMediaPropertiesChanged((error, media) => {
      !error && this._onMediaPropertiesChanged(media)
    })

    this.smtc.onTimelinePropertiesChanged((error, media) => {
      !error && this._onTimelinePropertiesChanged(media)
    })

    this.smtc.onPlaybackInfoChanged((error, media) => {
      !error && this._onPlaybackInfoChanged(media)
    })

    this.smtc.onSessionAdded((error, session) => {
      !error && this._onSessionAdded(session)
    })

    this.smtc.onSessionRemoved((error, sourceAppId) => {
      !error && this._onSessionRemoved(sourceAppId)
    })
  }

  _onMediaPropertiesChanged(media) {
    const { sourceAppId } = media
    const session = this._mediaSessions.get(sourceAppId)
    if (!session) {
      return
    }

    this._mediaSessions.set(sourceAppId, media)
    this.emit("session-media-changed", media)
  }

  _onTimelinePropertiesChanged(media) {
    const { sourceAppId } = media
    const session = this._mediaSessions.get(sourceAppId)
    if (!session) {
      return
    }

    this._mediaSessions.set(sourceAppId, media)
    this.emit("session-timeline-changed", media)
  }

  _onPlaybackInfoChanged(media) {
    const { sourceAppId } = media
    const session = this._mediaSessions.get(sourceAppId)
    if (!session) {
      return
    }

    this._mediaSessions.set(sourceAppId, media)
    this.emit("session-playback-changed", media)
  }

  _onSessionAdded(media) {
    const { sourceAppId } = media
    this._mediaSessions.set(sourceAppId, media)
    this.emit("session-added", media)
  }

  _onSessionRemoved(sourceAppId) {
    const mediaSession = this._mediaSessions.get(sourceAppId)
    if (mediaSession) {
      this._mediaSessions.delete(sourceAppId)
      this.emit("session-removed", sourceAppId)
    }
  }

  getAllMediaSessions() {
    return Array.from(this._mediaSessions.values())
  }

  static getCurrentMediaSessions() {
    return getAllSessions()
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
    warnings.push(
      `Please use Windows 10 version 1809 (10.0.17763) or later.`
    )
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
