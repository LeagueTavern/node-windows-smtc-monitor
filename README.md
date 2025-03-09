# Node-Windows-SMTC-Monitor

<a href="https://github.com/LeagueTavern/node-windows-smtc-monitor/actions"><img alt="GitHub CI Status" src="https://github.com/LeagueTavern/node-windows-smtc-monitor/workflows/CI/badge.svg?branch=master"></a>
<a href="https://www.npmjs.com/package/@coooookies/windows-smtc-monitor"><img src="https://img.shields.io/npm/v/@coooookies/windows-smtc-monitor.svg?sanitize=true" alt="@coooookies/windows-smtc-monitor npm version"></a>
<a href="https://npmcharts.com/compare/@coooookies/windows-smtc-monitor?minimal=true"><img src="https://img.shields.io/npm/dm/@coooookies/windows-smtc-monitor.svg?sanitize=true" alt="@coooookies/windows-smtc-monitor downloads"></a>

> node-windows-smtc-monitor is a [Node.js](https://nodejs.org/) toolkit for listening to [SMTC](https://learn.microsoft.com/en-us/uwp/api/windows.media.control.globalsystemmediatransportcontrolssessionmanager?view=winrt-26100) (System Media Transport Controls) media events in Windows. It is written in [Rust](https://www.rust-lang.org/) and utilizes [napi-rs](https://napi.rs/) to implement bindings with Node.js.

## Screenshot
![Screenshot](https://raw.githubusercontent.com/LeagueTavern/node-windows-smtc-monitor/master/docs/screenshot-1.png)

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

 [CommonJS Example](example/index.js)
 [ESModule Example](example/index.mjs)
 [TypeScript Example](example/index.ts)