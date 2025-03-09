# Node-Windows-SMTC-Monitor

<a href="https://github.com/LeagueTavern/node-windows-smtc-monitor/actions"><img alt="GitHub CI Status" src="https://github.com/LeagueTavern/node-windows-smtc-monitor/workflows/CI/badge.svg?branch=master"></a>
<a href="https://www.npmjs.com/package/@coooookies/windows-smtc-monitor"><img src="https://img.shields.io/npm/v/@coooookies/windows-smtc-monitor.svg?sanitize=true" alt="@coooookies/windows-smtc-monitor npm version"></a>
<a href="https://npmcharts.com/compare/@coooookies/windows-smtc-monitor?minimal=true"><img src="https://img.shields.io/npm/dm/@coooookies/windows-smtc-monitor.svg?sanitize=true" alt="@coooookies/windows-smtc-monitor downloads"></a>

> This is a [Node.js](https://nodejs.org/) toolkit for listening to [SMTC](https://learn.microsoft.com/en-us/uwp/api/windows.media.control.globalsystemmediatransportcontrolssessionmanager?view=winrt-26100) (System Media Transport Controls) media events in Windows. It is written in [Rust](https://www.rust-lang.org/) and utilizes [napi-rs](https://napi.rs/) to implement bindings with Node.js.

![Screenshot](docs/screenshot-1.png)

## ⚠️ Warning
`node-windows-smtc-monitor` only supports Windows 10 1809 and later versions (>= 10.0.17763)

## Features

- Listen to media events such as play, pause, next track, previous track.
- Get the current playback state and track information.
- Support for both JavaScript and TypeScript.
- Easy to use and integrate into existing Node.js applications.

## Installation

```shell
npm i @coooookies/windows-smtc-monitor
```

## Example

[CommonJS Example](example/index.js) <br />
[ESModule Example](example/index.mjs) <br />
[TypeScript Example](example/index.ts) <br />

## Usage

#### Importing the library

```Typescript
// Typescript & ESModule
import { SMTCMonitor } from '@coooookies/windows-smtc-monitor';

// Typescript Definition (Optional)
import type { MediaInfo, MediaProps, PlaybackInfo, TimelineProps } from "@coooookies/windows-smtc-monitor"

// CommonJS
const { SMTCMonitor } = require('@coooookies/windows-smtc-monitor');
```

#### Gets all media sessions

Gets all of the available sessions.

```Typescript
const sessions = SMTCMonitor.getMediaSessions(); // MediaSession[]
// [
//   {
//     sourceAppId: 'PotPlayerMini64.exe',
//     media: {
//       title: 'ぱられループ を歌ってみた (Jeku remix)',
//       artist: 'Jeku/aori',
//       albumTitle: '',
//       albumArtist: 'ぱられループ を歌ってみた (Jeku remix)',
//       genres: [],
//       albumTrackCount: 0,
//       trackNumber: 0,
//       thumbnail: <Buffer 42 4d 0e ... 1048526 more bytes> // The Album Cover/Thumbnail in Buffer
//     },
//     playback: { playbackStatus: 4, playbackType: 1 },
//     timeline: { position: 217.228, duration: 259 },
//     lastUpdatedTime: 1740000000000
//   },
//   {
//     sourceAppId: 'player.exe',
//     media: { ... },
//     playback: { ... },
//     timeline: { ... },
//     lastUpdatedTime: 1740000000000
//   }
// ]
```

#### Gets the current media session

Gets the current session. This is the session the system believes the user would most likely want to control.

```Typescript
const session = SMTCMonitor.getCurrentMediaSession(); // MediaSession | null
// {
//   sourceAppId: 'PotPlayerMini64.exe',
//   media: { ... },
//   playback: { ... },
//   timeline: { ... },
//   lastUpdatedTime: 1740000000000
// }
```

#### Gets the specified media session

Gets the specified session by the sourceAppId.

```Typescript
const session = SMTCMonitor.getMediaSessionByAppId('player.exe'); // MediaSession | null
// {
//   sourceAppId: 'player.exe',
//   media: { ... },
//   playback: { ... },
//   timeline: { ... },
//   lastUpdatedTime: 1740000000000
// }
```
