# fnOS HTTPS 桌面窗口白屏排障与修复（2.0.9 测试版）

> 重要：2.0.8 在真机安装后仍然显示空白窗口；2.0.9 对照官方示例再次修复入口及前端静态资源路径，但尚未得到用户设备的运行数据，因此不宣称已经实机修复。

## 2.0.9 新增的重点排查

- fnOS 官方网关入口应使用固定的 `/app/motrix/`，不要在安装入口中拼接 `?v=...`。
- FPK 内网页的 JS/CSS 必须从 `/app/motrix/assets/...` 加载，不能依赖相对路径，否则省略尾部斜杠时会错误地请求 `/app/assets/...`。
- 本项目仍保留 `http://NAS_IP:17080/` 作为管理服务备用入口，以便判断后端是否正常工作。
- 本地脚本 [`scripts/diagnose-fnos-window.sh`](../scripts/diagnose-fnos-window.sh) 可分别检查本机 TCP 端口、Unix Socket、Socket 内页面与资产以及飞牛统一网关入口。不会读取管理密码或修改数据库。

## 如果安装 2.0.9 后仍然是灰白色空窗口

首先在**登录了飞牛的同一浏览器**中直接打开 `<飞牛管理界面的协议和主机>/app/motrix/`（例如 `http://192.168.1.10:5666/app/motrix/`）。

- **404 或 9 字节 `not found`**：可能是 fnOS 应用服务网关未注册，不是下载器自己返回的页面。请在应用中心停止再启动应用，刷新飞牛网页；不要自行改写系统数据库。
- **502 Bad Gateway**：网关已找到路由但无法连接 `app.sock`，检查应用启动、Socket 所在路径/所有者以及 `/var/apps/motrix/var/logs/lifecycle.log`。不能只因 `17080` 正常就认定 Socket 正常。
- **正常 HTTP 200 但界面仍白屏**：在 F12 → Network 查看 `/app/motrix/assets/*.js` 和 `*.css` 是否 HTTP 200、内容类型分别为 JavaScript 和 CSS；同时检查 Console 中 CSP、混合内容、JS 报错。
- **直接 `/app/motrix/` 被重定向到登录**：未登录时正常。先登录 fnOS，再确认是否能在同一个站点加载页面。

在 NAS SSH 中运行（只读）：

```sh
sh /path/to/Motrix-FnOS/scripts/diagnose-fnos-window.sh
```

不方便获取脚本时，执行下面三条无修改命令，并把结果返回（IP/域名可打码）：

```sh
ls -ld /var/apps/motrix/target; ls -l /var/apps/motrix/target/app.sock
curl -sS -o /dev/null -w 'TCP 17080: %{http_code}\n' http://127.0.0.1:17080/
curl -sS --unix-socket /var/apps/motrix/target/app.sock -o /dev/null -w 'Unix Socket: %{http_code}\n' http://localhost/app/motrix/
```


## 版本与风险

这是 imaicai/Motrix-FnOS 基于 RockerHX/motrix-fnos 2.0.7 的独立测试 Fork，非官方发布。尚未在用户的 fnOS NAS 真机确认网关注册，不能保证已修复白屏。上游记录过 Socket 直连 HTTP 200、fnOS 网关却返回 `404 9` 的情况，因此特别保留独立 17080 入口以便回退。

## 为什么端口可用但飞牛桌面不可见

原版 `app/ui/config` 使用 `iframe + http + 17080`。当 NAS 管理桌面通过 HTTPS 加载 HTTP iframe 时，浏览器可能因混合内容拒绝展示；也可能是 fnOS 桌面应用入口未注册或权限有误。不能在没有浏览器控制台与真机日志的情况下把原因认定为唯一根因。

## 改动

1. FPK 桌面入口改用 `gatewayPrefix=/app/motrix` 和 `gatewaySocket=app.sock`，不再使用 HTTP iframe 端口配置。
2. Rust 服务在 `${TRIM_APPDEST}/app.sock` 启动 Unix Socket，再将请求透明转发到本机管理端口 `127.0.0.1:17080`。Socket 初始化失败时记录错误，17080 独立入口仍运行。
3. 前端在 `/app/motrix/` 下使用同源 API 和 SSE 路径，兼容 fnOS 网关剥离或保留前缀；Vite 静态资源继续使用相对路径。
4. 保留 `service_port=17080`、原 Motrix 管理密码/JWT、JSON-RPC 独立端口及 SQLite 数据库结构；未修改下载引擎任务逻辑。

## 安装包与构建

- 在 [Releases](https://github.com/imaicai/Motrix-FnOS/releases) 选择已成功生成的 **预发布测试包**，不要安装 GitHub Actions 失败/未完成时的文件。
- 设备为 Intel/AMD x86_64 时选 `_x86.fpk`，ARM64/aarch64 时选 `_arm.fpk`。
- 安装前停止当前下载任务，备份 Motrix 应用私有数据、数据库、配置、session、密码与下载目录。**不要先卸载并删除数据**。
- 测试包使用原 `appname=motrix`、版本 2.0.9，设计上支持从原版 2.0.7 升级并保留数据；这仍有环境兼容风险。
- 源码构建要求 Node 24+、pnpm、Rust 1.99.1、cargo-zigbuild、Zig 和 fnpack。依次运行 `pnpm install --frozen-lockfile`、`pnpm run verify`、`pnpm run build:fpk:artifacts`、`pnpm run verify:fpk`。

## 真机验收步骤

1. 通过 `http://NAS_IP:17080/` 确认原独立管理入口可以正常登录。
2. 在飞牛 HTTPS 网页桌面双击 Motrix 应用，确认登录页、任务列表和下载进度不再空白。
3. 在 NAS Shell 查 Socket：`ls -l /var/apps/motrix/target/app.sock`；这是假设的安装路径，实际以 `TRIM_APPDEST` 为准。
4. 检查本机 Socket：`curl --unix-socket /var/apps/motrix/target/app.sock http://localhost/app/motrix/api/app/ping`。
5. 对比飞牛 HTTPS `https://NAS_HOST:HTTPS_PORT/app/motrix/` 是否可访问。如果显示 `404 9`，优先调查系统应用网关注册；如果 502，调查 Socket 路径、权限和代理转发。
6. 登录后检查 API、SSE 进度自动刷新、新增下载任务、授权目录、回收站、手机飞牛 WebView。
7. 停止和重新启动应用，再次检查 Socket、HTTPS 内嵌和独立端口。

## 安全与回退

由于 fnOS 网关 UID/GID 尚未明确，Socket 的测试权限暂设为 `0666`，但只能回源到原本向局域网开放、仍要求管理密码/JWT 的 17080。正式生产版应核验网关用户，尽量收紧为 0660 与正确属组。

如果出现新的故障，保留浏览器控制台日志、fnOS 应用日志、Socket 权限和网关 HTTP 状态；恢复与原版版本配套的 FPK 及备份，而非混用新旧数据库。

原作者及项目资料：[RockerHX/motrix-fnos](https://github.com/RockerHX/motrix-fnos)。
