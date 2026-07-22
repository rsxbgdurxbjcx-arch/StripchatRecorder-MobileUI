# StripchatRecorder-MobileUI

[简体中文](README.md) | [English](README.en.md)

A self-hosted Stripchat live stream recorder with a web-based management UI. Supports automatic recording, post-processing pipelines, and multi-channel notifications.

> This project is forked from [ChanTrail/StripchatRecorder](https://github.com/ChanTrail/StripchatRecorder), with the following additions:
> - **Android mobile UI adaptation**: bottom navigation bar (with icons), 2-column streamer cards, compact recordings table, fixed page size
> - **Login system**: initial account `sr-mobileui` / password `admin`, changeable in settings
> - **Telegram auto-delete local files**: optionally deletes local videos after successful upload or on upload failure

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-3.0.html)

---

## Features

- Monitor multiple streamers and auto-record when they go live
- Web UI for managing streamers, recordings, and post-processing
- **Android mobile UI adaptation**: fixed bottom menu (with icons, pinned in document flow so it never shifts), 2-column streamer cards, compact recordings table, toast notifications not blocking bottom nav
- **Login system**: token-based authentication to protect the Web UI from unauthorized access
- **Streamer Finder**: discover streamers via [camgirlfinder.net](https://camgirlfinder.net)
- **HLS Relay**: proxy a streamer's live stream to any player without recording
- Supports split network proxies: configure Stripchat API proxy and CDN chunk proxy separately
- **Mouflon HLS decryption**: manage `pkey → pdkey` key pairs
- Configurable post-processing pipeline with pluggable modules:
  - **contact_sheet** — generates a tiled preview image with timestamps
  - **filter_short** — deletes recordings below a minimum duration
  - **notify_discord** — sends recording info and cover image to a Discord Webhook
  - **notify_telegram** — sends recording info, cover image, and video via MTProto (**supports auto-delete local file after upload**)
- Disk space monitoring on the recordings page
- Real-time UI updates via Server-Sent Events with multi-client sync; silent data refresh after reconnect without a full page reload
- Dark/light mode following system theme (pure white background in light mode)

---

## Quick Start (Docker)

### Deployment

```bash
git clone https://github.com/rsxbgdurxbjcx-arch/StripchatRecorder-MobileUI.git
cd StripchatRecorder-MobileUI
docker compose up -d
```

After startup, open `http://localhost:4040` in your browser and log in with the initial account `sr-mobileui` / password `admin`.

> **Deployment speed**: This project pulls the pre-built Docker image `chantrail/stripchat-recorder:latest` from Docker Hub for the runtime environment, and injects the custom backend binary (with mobile UI + login system) and the modified `notify_telegram` module binary via volume mount. The entire deployment process typically completes within **2 minutes**.

The Docker image runs in Server mode (port 4040), with configuration written to the mounted `config/settings.json`.

`docker-compose.yml` configuration (included in the repository):

```yaml
services:
  stripchat-recorder:
    image: chantrail/stripchat-recorder:latest
    container_name: stripchat-recorder
    restart: unless-stopped
    environment:
      - TZ=Asia/Shanghai
      # - LANGUAGE=en-US  # Set interface language: zh-CN (default) or en-US
    ports:
      - "4040:3030"
    volumes:
      - ./data/logs:/app/stripchat-recorder/logs
      - ./data/recordings:/app/stripchat-recorder/recordings
      - ./data/modules:/app/stripchat-recorder/modules
      - ./data/config:/app/stripchat-recorder/config
      # Override the in-image backend binary with custom build (mobile UI + login system)
      - ./data/bin/stripchat-recorder:/app/stripchat-recorder/stripchat-recorder
```

### Main Settings

The following options can be configured in the Web UI "Settings" page:

| Setting | Description |
|---------|-------------|
| Output directory | Recording file save path |
| Max concurrent recordings | Maximum number of simultaneous recordings, `0` = unlimited |
| Poll interval | Interval to check if streamers are online (seconds), range 10–300 |
| Merge format | Format for auto-merging segments after recording: `mp4` (default), `mkv`, `ts` |
| Auto-record on live | Whether new streamers have auto-record enabled by default |
| Max temp dir usage | Maximum temporary file usage for post-processing modules (GB), `0` = unlimited, default 50 GB |

### Network Proxies and Mirror

In the "Network" section of the settings page, you can configure the API proxy, CDN proxy, and Stripchat mirror separately.

### Mouflon HLS Decryption Keys

Stripchat encrypts HLS segment filenames (Mouflon system). If you encounter issues downloading segments during recording, you need to fill in the corresponding `pkey → pdkey` key pairs in the "Mouflon Decryption Keys" section of the settings page.

### HLS Relay

In Server mode, you can directly open the following URL in a player to play the live stream:

```
http://localhost:4040/stream/{modelname}
```

---

## New Features

### 1. Android Mobile UI Adaptation

- **Bottom navigation bar**: On mobile (< 768px), the desktop sidebar is hidden and a bottom navigation bar with functional icons is displayed
- **2-column streamer cards**: On mobile, the streamer list displays 2 cards per row, with compact card height and fully visible thumbnails
- **Compact recordings table**: On mobile, the recordings table uses smaller font size, reduced spacing, and adapts to narrow screens
- **Toast notifications not blocking**: Toast notifications are moved up to avoid blocking the bottom navigation bar
- **Fixed bottom menu**: The app shell is pinned to the viewport via `position: fixed`, and its height is synced in real time using the `visualViewport` API (fallback `100dvh`), so on Android Chrome **and Firefox** — through address bar resizing, overscroll, and keyboard pop-ups — the bottom menu stays firmly pinned and never comes loose or disappears
- **Input zoom prevention**: Mobile input fields use 16px font size to prevent Android Chrome focus zoom
- **Post-processing input alignment**: All module input fields are aligned with uniform height

### 2. Login System

- **Initial account**: `sr-mobileui` / password: `admin`
- **Authentication mechanism**: Token-based authentication, all API requests carry `Authorization: Bearer {token}` after login
- **SSE authentication**: SSE connections pass tokens via query parameters (EventSource does not support custom headers)
- **Change password**: In the "Settings" page, you can change the account and password. After changing, you are automatically logged out and need to re-login
- **Persistence**: Account and password are stored in `config/auth.json` (password hashed with SHA-256)
- **Security**: After changing the password, all logged-in tokens are invalidated, forcing re-login

### 3. Telegram Auto-Delete Local Files

In the Web UI "Post-processing" page, when editing the `notify_telegram` node parameters, you can see two auto-delete switches:

- **Delete local file after upload** (default off): once enabled, the local video file and its metadata are deleted immediately after a successful Telegram upload
- **Delete local file on upload failure** (default off): once enabled, the local video file is also deleted when the upload still fails after multiple retries (useful when disk space is tight and failed files are not worth keeping); the post-processing status is still truthfully marked as failed

**Thorough deletion guarantee**: auto-deletion now removes every file associated with the recording, leaving nothing behind on disk:

- The video file + metadata (meta) + sidecar cover images in the same directory (webp/jpg/jpeg/png)
- Module output files recorded in the meta (e.g. large preview images generated by contact_sheet)
- All temp files produced by the module: split segments (full-size video copies), format-conversion copies, video thumbnails, resized covers — cleaned up before the module exits on BOTH success and failure paths
- **Every time** "delete after upload" or "delete on failure" fires, orphaned temp files (including `_split_tmp_*` dirs) are pruned at the same time
- **Every 1 minute** a tmp cleanup fires, but it only clears the tmp directory (including `_split_tmp_*` dirs) **when no post-processing task is running** — in-flight upload/split temp files are never touched, so uploads of large and small files are never affected; as soon as post-processing goes idle the tmp dir is fully cleared, so orphaned files survive at most ~1 minute and the disk can never be silently filled
- On every startup the module prunes orphaned temp files older than 6 hours in the tmp directory (full-size video copies left behind by killed/cancelled/crashed processes — the main cause of "phantom" disk usage)
- When the host fails to delete a file, it retries 3 times (500ms apart) and logs an error if it still fails

Also fixed the "upload succeeded but local file not deleted" issue: previously, if the delete switch was off at upload time or the host's deletion silently failed, the leftover file would never be processed again because re-runs skip already-succeeded modules. Now re-running post-processing performs a fallback check: any file whose node has "delete after upload" enabled and whose upload already succeeded is deleted directly, without re-uploading.

---

## Post-processing Modules

Modules are standalone executables that implement a simple protocol. The repository includes the pre-compiled binary for the modified `notify_telegram` module in `data/modules/notify_telegram_v030`. The other three modules are provided by the Docker image.

---

## Project Structure

```
StripchatRecorder-MobileUI/
├── Dockerfile                  # Build definition of the pre-built image (Docker Hub: chantrail/stripchat-recorder)
├── docker-compose.yml          # Deployment entry: pre-built image + volume-injected pre-compiled binaries
├── package.json                # Root build scripts (npm run build for a full build)
├── backend/                    # Rust backend (Axum, frontend assets embedded)
│   └── src/
│       ├── main.rs             # Executable entry (Server mode)
│       ├── server_mod/         # HTTP routes, SSE, API handlers, periodic tasks (incl. 1-minute tmp cleanup)
│       ├── commands/           # Business commands (recording/post-processing/settings/streamers)
│       ├── postprocess/        # Post-processing pipeline engine (DELETE_INPUT protocol, thorough deletion, tmp cleanup)
│       ├── recording/          # Recorder, merging, meta metadata
│       ├── streaming/          # Stripchat monitoring and HLS pulling
│       ├── relay/              # HLS relay
│       ├── watcher/            # Directory watchers (recordings/modules/locales)
│       ├── config/             # Settings and global state
│       ├── core/               # Logging, auth, event broadcast, error types
│       └── locale/             # i18n management and default locale files
├── frontend/                   # Vue 3 + TypeScript + Vite + Tailwind frontend (mobile adapted)
│   └── src/
│       ├── views/              # Pages: streamers/recordings/post-processing/relay/finder/settings
│       ├── stores/             # Pinia stores
│       ├── composables/        # Composables
│       ├── components/         # UI components
│       └── locales/            # Frontend zh-CN / en-US language packs
├── modules/                    # Post-processing modules (standalone Rust binaries)
│   ├── notify_telegram/        # Telegram notification (MTProto upload, two auto-delete switches)
│   ├── contact_sheet/          # Thumbnail contact sheet
│   ├── filter_short/           # Short-file filter
│   ├── notify_discord/         # Discord notification
│   └── pp_utils/               # Shared module library (param parsing, tmp dir, ffmpeg helpers)
├── desktop/                    # Tauri desktop app (shares the frontend; not needed for Docker deployment)
├── data/
│   ├── bin/stripchat-recorder  # Pre-compiled backend binary (injected into the image at deploy time)
│   └── modules/notify_telegram_v030  # Pre-compiled Telegram module binary (injected at deploy time)
├── scripts/                    # Build/check/release scripts
└── docs/                       # Module development, custom locale docs, etc.
```

> Runtime data (`data/recordings`, `data/logs`, `data/config`) is created via docker-compose volume mounts and is not part of the source tree.

---

## Tech Stack

- **Frontend:** Vue 3, TypeScript, Vite, Tailwind CSS, Reka UI
- **Backend:** Rust, Axum
- **Post-processing modules:** Rust (standalone binaries)
- **Container:** Debian, ffmpeg

---

## Acknowledgments

This project is based on [ChanTrail/StripchatRecorder](https://github.com/ChanTrail/StripchatRecorder). Thanks to the original author for their contribution.

---

## License

This project is released under the [GNU General Public License v3.0](https://www.gnu.org/licenses/old-licenses/gpl-3.0.html).

---

## Disclaimer

This project is for technical research and learning purposes only. Users bear all risks related to deployment, operation, and compliance.
