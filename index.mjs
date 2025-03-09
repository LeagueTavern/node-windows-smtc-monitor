import { createRequire } from "module"
const require = createRequire(import.meta.url)
export const { SMTCMonitor, PlaybackStatus } = require("./index.js")
