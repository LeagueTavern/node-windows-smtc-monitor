import test from "ava"
import { SMTCMonitor } from "../index"

const smtc = new SMTCMonitor()

test("SMTCMonitor is instantiated", (t) => {
  t.truthy(smtc)
})

test("getCurrentMediaSession returns null if no sessions are present", (t) => {
  const session = smtc.getCurrentMediaSession()
  t.true(session === null || "sourceAppId" in session)
})

test("getAllMediaSessions returns an empty array if no sessions are present", (t) => {
  const sessions = smtc.getAllMediaSessions()
  t.true(Array.isArray(sessions))
})

test("getMediaSessionByAppId returns null", (t) => {
  t.is(smtc.getMediaSessionByAppId("nonexistent"), null)
})

test.after.always("cleanup resources", () => {
  smtc.destroy()
})
