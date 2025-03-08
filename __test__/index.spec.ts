import test from "ava"
import { getCurrentSession, getAllSessions, getSessionById } from "../binding"

test("getCurrentSession() should return MediaInfo | null", (t) => {
  const session = getCurrentSession()
  t.true(session === null || "sourceAppId" in session)
})

test("getAllSessions() should return MediaInfo[]", (t) => {
  const sessions = getAllSessions()
  t.true(Array.isArray(sessions))
})

test("getSessionById() should return null", (t) => {
  t.is(getSessionById("nonexistent"), null)
})
