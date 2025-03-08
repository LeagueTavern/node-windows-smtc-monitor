/**
 * Global System Media Transport Controls Session Playback Status
 * @enum {number}
 */
const PlaybackStatus = {
  CLOSED: 0,
  OPENED: 1,
  CHANGING: 2,
  STOPPED: 3,
  PLAYING: 4,
  PAUSED: 5,
}

module.exports = {
  PlaybackStatus,
}
