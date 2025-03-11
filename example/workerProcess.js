const { parentPort } = require("worker_threads")
const { SMTCMonitor } = require("..")

const port = parentPort
if (!port) throw new Error("IllegalState")

const smtc = new SMTCMonitor()

port.on("message", (message) => {
  switch (message.type) {
    case "getCurrentSession":
      port.postMessage({
        type: "result",
        result: SMTCMonitor.getCurrentMediaSession(),
      })
      break
    case "getSessions":
      port.postMessage({
        type: "result",
        result: SMTCMonitor.getMediaSessions(),
      })
      break
    case "getSessionById":
      port.postMessage({
        type: "result",
        result: SMTCMonitor.getMediaSessionByAppId(message.sessionId),
      })
      break
    case "sessions":
      port.postMessage({
        type: "sessions",
        result: smtc.sessions,
      })
      break
    case "invoke": {
      const { event, args, symbol } = message
      try {
        const result = args ? SMTCMonitor[event](...args) : SMTCMonitor[event]()
        port.postMessage({ event: "invoke-result", symbol, result })
      } catch (error) {
        port.postMessage({
          event: "invoke-result",
          symbol,
          error,
        })
      }
      break
    }
    default:
      port.postMessage({ type: "error", error: "Unknown message type" })
  }
})

smtc.on("current-session-changed", (sourceAppId) => {
  port.postMessage({ type: "current-session-changed", sourceAppId })
})

smtc.on("session-added", (sourceAppId, mediaProps) => {
  port.postMessage({ type: "session-added", sourceAppId, mediaProps })
})

smtc.on("session-removed", (sourceAppId) => {
  port.postMessage({ type: "session-removed", sourceAppId })
})

smtc.on("session-media-changed", (sourceAppId, mediaProps) => {
  port.postMessage({ type: "session-media-changed", sourceAppId, mediaProps })
})

smtc.on("session-timeline-changed", (sourceAppId, timelineProps) => {
  port.postMessage({
    type: "session-timeline-changed",
    sourceAppId,
    timelineProps,
  })
})

smtc.on("session-playback-changed", (sourceAppId, playbackInfo) => {
  port.postMessage({
    type: "session-playback-changed",
    sourceAppId,
    playbackInfo,
  })
})
