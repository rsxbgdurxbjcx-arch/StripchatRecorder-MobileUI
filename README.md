# StripchatRecorder-MobileUI

[简体中文](README.md) | [English](README.en.md)

自托管的 Stripchat 直播录制工具，提供基于 Web 的管理界面，支持自动录制、后处理流水线和多渠道通知。

> 本项目 Fork 自 [ChanTrail/StripchatRecorder](https://github.com/ChanTrail/StripchatRecorder)，在原版基础上新增了：
> - **全新 UI 设计**：暖调 Noir Crimson 主题（深/浅色随系统自动切换）、绯红主色、细腻投影与按压反馈、毛玻璃导航
> - **安卓移动端全面适配**：底部导航栏（带图标 + 安全区适配）、主播卡片 2 列极致紧凑布局、录制表格独立滚动 + 图标化操作列、固定页面大小
> - **登录系统**：初始账号 `sr-mobileui` / 密码 `admin`，支持在设置中修改账号密码
> - **Telegram 自动删除本地文件**：支持上传成功后 / 上传失败时自动清理本地视频

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-3.0.html)

---

## 功能特性

- 监控多个主播，上线时自动开始录制
- Web UI 管理主播、录制文件和后处理任务
- **安卓移动端全面适配**：固定底部菜单（带图标，文档流钉底永不移位）、主播卡片 2 列、录制表格紧凑、提示反馈不遮挡菜单
- **登录系统**：基于 token 的认证，保护 Web UI 不被未授权访问
- **主播查找**：通过 [camgirlfinder.net](https://camgirlfinder.net) 查找主播
- **转发流（HLS Relay）**：无需录制即可将主播直播流转发给播放器
- 支持分离式网络代理：可分别配置 Stripchat API 代理与 CDN 分片代理
- **Mouflon HLS 解密**：支持管理 `pkey → pdkey` 密钥对
- 可配置的后处理流水线，支持插件化模块：
  - **contact_sheet** — 生成带时间戳的缩略图预览图
  - **filter_short** — 删除低于最短时长的录制文件
  - **notify_discord** — 通过 Discord Webhook 发送录制信息和封面图
  - **notify_telegram** — 通过 MTProto 发送录制信息、封面图和视频（**支持上传后自动删除本地文件**）
- 录制文件页磁盘空间监控
- 基于 SSE 的实时 UI 更新，支持多客户端同步；断线重连后无感刷新数据，无需整页重载
- 跟随系统主题的深色/浅色模式（浅色为纯白背景）

---

## 快速开始（Docker）

### 部署方式

```bash
git clone https://github.com/rsxbgdurxbjcx-arch/StripchatRecorder-MobileUI.git
cd StripchatRecorder-MobileUI
docker compose up -d
```

启动后在浏览器中打开 `http://localhost:4040`，使用初始账号 `sr-mobileui` / 密码 `admin` 登录。

> **部署速度说明**：本项目拉取 Docker Hub 上的预构建镜像 `chantrail/stripchat-recorder:latest` 提供运行环境，通过 volume mount 注入自定义后端二进制（含移动端适配 + 登录系统）和修改后的 `notify_telegram` 模块二进制。整个部署过程通常在 **2 分钟内**完成。

Docker 镜像以 Server 模式运行（端口 4040），配置写入挂载的 `config/settings.json`。

### 主要设置项

在 Web UI 的「设置」页面可配置以下选项：

| 设置项 | 说明 |
|--------|------|
| 输出目录 | 录制文件保存路径 |
| 最大并发录制数 | 同时录制的最大主播数，`0` 表示不限制 |
| 轮询间隔 | 检查主播是否上线的间隔（秒），范围 10–300 |
| 合并格式 | 录制结束后自动合并分片的格式：`mp4`（默认）、`mkv`、`ts` |
| 上线自动录制 | 新添加的主播是否默认开启自动录制 |
| 后处理临时目录最大占用 | 后处理模块运行时产生的临时文件上限（GB），超出后自动删除最旧的文件，`0` 表示不限制，默认 50 GB |

### 网络代理与镜像站

在设置页的「网络」中可分别配置 API 代理、CDN 代理和 Stripchat 镜像站。

### Mouflon HLS 解密密钥

Stripchat 对 HLS 分片文件名进行了加密（Mouflon 系统）。若录制时遇到无法下载分片的情况，需在设置页的「Mouflon 解密密钥」中填入对应的 `pkey → pdkey` 密钥对。

### 转发流（HLS Relay）

在 Server 模式下，直接用播放器打开以下地址即可播放直播：

```
http://localhost:4040/stream/{modelname}
```

---

## 新增功能详解

### 1. 安卓移动端全面适配

- **底部导航栏**：移动端（< 768px）自动隐藏桌面侧边栏，显示底部导航栏，每个菜单带功能图标
- **主播卡片 2 列**：移动端主播列表一行展示 2 个卡片，卡片高度紧凑，缩略图完整显示
- **录制表格紧凑**：移动端录制文件表格缩小字体、减小间距，适配窄屏
- **提示反馈不遮挡**：toast 提示上移，不遮挡底部导航栏
- **固定底部菜单**：应用外壳通过 `position: fixed` 钉在视口，并借助 `visualViewport` API 实时同步外壳高度（回退 `100dvh`），安卓 Chrome **和 Firefox** 的地址栏伸缩、过度滚动、键盘弹起时底部菜单都牢牢固定、永不松动或消失
- **输入框防缩放**：移动端输入框字体 16px，防止安卓 Chrome 聚焦缩放
- **后处理输入框对齐**：所有模块的输入框统一高度对齐

### 2. 登录系统

- **初始账号**：`sr-mobileui` / 密码：`admin`
- **认证机制**：基于 token 的认证，登录后所有 API 请求携带 `Authorization: Bearer {token}`
- **SSE 认证**：SSE 连接通过查询参数传递 token（EventSource 不支持自定义 header）
- **修改密码**：在「设置」页面底部可修改账号和密码，修改后自动退出重新登录
- **持久化**：账号密码存储在 `config/auth.json`（密码用 SHA-256 哈希）
- **安全**：修改密码后所有已登录 token 失效，强制重新登录

### 3. Telegram 自动删除本地文件

在 Web UI 的「后处理」页面，编辑 `notify_telegram` 节点参数时，可看到两个自动删除开关：

- **上传后自动删除本地文件**（默认关闭）：开启后，视频文件上传 Telegram 成功后，立即自动删除本地视频文件及其元数据
- **上传失败自动删除本地文件**（默认关闭）：开启后，多次重试上传仍失败时也会删除本地视频文件（适合磁盘紧张、失败文件无需保留的场景），后处理状态仍会如实标记为失败

**彻底删除保障**：自动删除现在会一并清除与该录制关联的所有文件，杜绝磁盘残留：

- 视频文件 + 元数据（meta）+ 同目录 sidecar 封面图（webp/jpg/jpeg/png）
- meta 中记录的模块输出文件（如 contact_sheet 生成的大预览图）
- 模块运行产生的全部临时文件：分割片段（整份视频副本）、格式转换副本、视频缩略图、缩放封面——成功与失败路径都会在模块退出前统一清理
- **每次触发**「上传后自动删除」或「上传失败自动删除」时，同步自动清理孤儿临时文件（含 `_split_tmp_*` 目录）
- **每 1 分钟**触发一次 tmp 清空任务，但**仅在没有后处理任务运行时**才全量清空（含 `_split_tmp_*` 目录）——上传/分割进行中的临时文件一根手指都不碰，大文件小文件上传互不影响；后处理一空闲立即清空，孤儿文件最多存活约 1 分钟，磁盘不会被临时文件悄悄占满
- 模块每次启动时自动清理 tmp 目录中 6 小时前的孤儿临时文件（被杀死/取消/崩溃的进程遗留的整份视频副本，这是磁盘空间"凭空"消失的主因）
- 主程序删除文件失败时自动重试 3 次（间隔 500ms），仍失败则记录错误日志

同时修复了「上传成功但本地文件未删除」的问题：此前若上传成功时删除开关未开启、或主程序删除文件静默失败，残留文件在重新后处理时会因「跳过已成功模块」而永远不再被处理。现在重新后处理时会兜底检查：凡开启了「上传后自动删除」且上传已成功的文件直接补删，不会重复上传。

---

## 后处理模块

模块是实现了简单协议的独立可执行文件。仓库 `data/modules/notify_telegram_v030` 已包含修改后的预编译二进制。其余三个模块由 Docker 镜像内置提供。

---

## 项目结构

```
StripchatRecorder-MobileUI/
├── Dockerfile                  # 预构建镜像的构建定义（Docker Hub: chantrail/stripchat-recorder）
├── docker-compose.yml          # 部署入口：预构建镜像 + volume 注入预编译二进制
├── package.json                # 根构建脚本（npm run build 全量构建）
├── backend/                    # Rust 后端（Axum，内嵌前端静态资源）
│   └── src/
│       ├── main.rs             # 可执行入口（Server 模式）
│       ├── server_mod/         # HTTP 路由、SSE、API  handlers、定时任务（含每 1 分钟清空 tmp）
│       ├── commands/           # 业务命令（录制/后处理/设置/主播）
│       ├── postprocess/        # 后处理流水线引擎（DELETE_INPUT 协议、彻底删除、tmp 清理）
│       ├── recording/          # 录制器、合并、meta 元数据
│       ├── streaming/          # Stripchat 监控与 HLS 拉流
│       ├── relay/              # HLS 转发流
│       ├── watcher/            # 目录监听（录制目录/模块目录/语言目录）
│       ├── config/             # 配置与全局状态
│       ├── core/               # 日志、认证、事件广播、错误类型
│       └── locale/             # 多语言管理与默认语言文件
├── frontend/                   # Vue 3 + TypeScript + Vite + Tailwind 前端（移动端适配）
│   └── src/
│       ├── views/              # 主播/录制文件/后处理/转发流/查找/设置等页面
│       ├── stores/             # Pinia 状态
│       ├── composables/        # 组合式函数
│       ├── components/         # UI 组件
│       └── locales/            # 前端中英文语言包
├── modules/                    # 后处理模块（独立 Rust 二进制）
│   ├── notify_telegram/        # Telegram 通知（MTProto 上传，支持两个自动删除开关）
│   ├── contact_sheet/          # 缩略图预览图
│   ├── filter_short/           # 短文件过滤
│   ├── notify_discord/         # Discord 通知
│   └── pp_utils/               # 模块公共库（参数解析、tmp 目录、ffmpeg 工具）
├── desktop/                    # Tauri 桌面端（与本项目共用前端，非 Docker 部署所需）
├── data/
│   ├── bin/stripchat-recorder  # 预编译后端二进制（部署时注入镜像）
│   └── modules/notify_telegram_v030  # 预编译 Telegram 模块二进制（部署时注入镜像）
├── scripts/                    # 构建/检查/发布脚本
└── docs/                       # 模块开发、自定义语言等文档
```

> 运行时数据（`data/recordings`、`data/logs`、`data/config`）由 docker-compose 的 volume 挂载生成，不包含在源码中。

---

## 技术栈

- **前端：** Vue 3, TypeScript, Vite, Tailwind CSS, Reka UI
- **后端：** Rust, Axum
- **后处理模块：** Rust（独立二进制）
- **容器：** Debian, ffmpeg

---

## 致谢

本项目基于 [ChanTrail/StripchatRecorder](https://github.com/ChanTrail/StripchatRecorder) 开发，感谢原作者的贡献。

---

## 开源许可证

本项目基于 [GNU 通用公共许可证 v3.0](https://www.gnu.org/licenses/old-licenses/gpl-3.0.html) 发布。

---

## 免责声明

本项目仅用于技术研究与学习交流。使用者需自行承担部署、运维与合规风险。
