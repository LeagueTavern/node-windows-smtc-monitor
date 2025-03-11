const crypto = require("crypto")
const { Worker } = require("worker_threads")

const worker = new Worker("./example/workerProcess.js")

function invoke(event, args) {
  const symbol = crypto.randomBytes(16).toString("hex")

  return new Promise((resolve, reject) => {
    const handle = ({ event, symbol: _symbol, result, error }) => {
      if (event !== "invoke-result" || _symbol !== symbol) return

      if (error) {
        reject(error)
      } else {
        resolve(result)
      }

      worker.off("message", handle)
    }

    worker.on("message", handle)
    worker.postMessage({ type: "invoke", event, args, symbol })
  })
}

worker.on("message", (result) => {
  if (result.event === "invoke-result") {
    return
  }

  console.log("message", result)
})

invoke("getMediaSessions").then((result) => {
  console.log("getMediaSessions", result)
})

invoke("getCurrentMediaSession").then((result) => {
  console.log("getCurrentMediaSession", result)
})

invoke("getMediaSessionByAppId", ["player.exe"]).then((result) => {
  console.log("getMediaSessionByAppId", result)
})
