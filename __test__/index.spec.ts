import test from "ava"
import { getCurrentSession, getSessions, getSessionById } from "../binding"

test("getCurrentSession() should return MediaInfo | null", (t) => {
  const session = getCurrentSession()
  t.true(session === null || "sourceAppId" in session)
})

test("getSessions() should return MediaInfo[]", (t) => {
  const sessions = getSessions()
  t.true(Array.isArray(sessions))
})

test("getSessionById() should return null", (t) => {
  t.is(getSessionById("nonexistent"), null)
})
