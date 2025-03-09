import { SMTCMonitor } from "../index.mjs"

const main = () => {
  console.log("---CURRENT MEDIA SESSION---")
  console.log(SMTCMonitor.getCurrentMediaSession())
  console.log("---ALL MEDIA SESSIONS---")
  console.log(SMTCMonitor.getCurrentMediaSessions())
  console.log("---[ player.exe ]---")
  console.log(SMTCMonitor.getMediaSessionByAppId("player.exe"))
  console.log("SMTC MONITOR IS LISTENING FOR EVENTS...")
}

const smtc = new SMTCMonitor()

smtc.on("session-media-changed", (appId, mediaProps) => {
  console.log(
    "session-media-changed",
    appId,
    mediaProps.title,
    mediaProps.thumbnail?.length
  )
})

smtc.on("session-timeline-changed", (appId, timelineProps) => {
  console.log(
    "session-timeline-changed",
    appId,
    `${
      timelineProps.duration > 0
        ? Math.round(
            (timelineProps.position / timelineProps.duration) * 10000
          ) / 100
        : "-"
    }%`,
    timelineProps.position,
    timelineProps.duration
  )
})

smtc.on("session-playback-changed", (appId, playbackInfo) => {
  console.log("session-playback-changed", appId, playbackInfo.playbackStatus)
})

smtc.on("session-added", (appId, mediaInfo) => {
  console.log("session-added", appId, mediaInfo.lastUpdatedTime)
})

smtc.on("session-removed", (appId) => {
  console.log("session-removed", appId)
})
main()
