import { SMTCMonitor } from "../index"

const smtc = new SMTCMonitor()

const main = () => {
  console.log("---CURRENT MEDIA SESSION---")
  console.log(SMTCMonitor.getCurrentMediaSession())
  console.log("---ALL MEDIA SESSIONS---")
  console.log(SMTCMonitor.getCurrentMediaSessions())
  console.log("SMTC MONITOR IS LISTENING FOR EVENTS...")
}

smtc.on("session-media-changed", (media) => {
  console.log(
    "session-media-changed",
    media.sourceAppId,
    media.title,
    media.thumbnail?.length
  )
})

smtc.on("session-timeline-changed", (media) => {
  console.log(
    "session-timeline-changed",
    media.sourceAppId,
    `${Math.round((media.position / media.duration) * 10000) / 100}%`,
    media.position,
    media.duration
  )
})

smtc.on("session-playback-changed", (media) => {
  console.log(
    "session-playback-changed",
    media.sourceAppId,
    media.playbackStatus
  )
})

smtc.on("session-added", (media) => {
  console.log("session-added", media.sourceAppId)
})

smtc.on("session-removed", (sourceAppId) => {
  console.log("session-removed", sourceAppId)
})

main()
