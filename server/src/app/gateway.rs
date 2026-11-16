//! fnOS 统一网关：Unix Socket -> 本机管理 HTTP 端口字节流代理。
use std::env;
use std::fs;
use std::net::{Ipv4Addr, SocketAddr};
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::PathBuf;
use tokio::io::copy_bidirectional;
use tokio::net::{TcpStream, UnixListener};
use tokio::task::JoinHandle;

const GATEWAY_SOCKET_ENV: &str = "MOTRIX_FNOS_GATEWAY_SOCKET";

pub(super) struct GatewayProxy {
    socket_path: PathBuf,
    socket_inode: u64,
    worker: JoinHandle<()>,
}

impl GatewayProxy {
    pub(super) fn start(management_addr: SocketAddr) -> Result<Option<Self>, String> {
        let Some(path) = env::var_os(GATEWAY_SOCKET_ENV).filter(|v| !v.is_empty()) else {
            return Ok(None);
        };
        let socket_path = PathBuf::from(path);
        match fs::symlink_metadata(&socket_path) {
            Ok(meta) if meta.file_type().is_socket() => {
                fs::remove_file(&socket_path).map_err(|e| format!("清理旧网关 Socket 失败：{e}"))?;
            }
            Ok(_) => return Err("网关 Socket 位置有非 Socket 文件，不覆盖".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("检查网关 Socket 失败：{e}")),
        }
        let listener = UnixListener::bind(&socket_path)
            .map_err(|e| format!("绑定 fnOS 网关 Socket 失败：{e}"))?;
        // 网关运行 UID 未经真机确认，测试阶段用 0666；管理 API 保持 JWT 鉴权。
        fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o666))
            .map_err(|e| format!("设置网关 Socket 权限失败：{e}"))?;
        let socket_inode = fs::symlink_metadata(&socket_path)
            .map_err(|e| format!("读取网关 Socket 信息失败：{e}"))?.ino();
        let upstream = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), management_addr.port());
        let worker = tokio::spawn(async move {
            loop {
                let (mut incoming, _) = match listener.accept().await {
                    Ok(pair) => pair,
                    Err(e) => { tracing::error!("fnOS gateway accept failed: {e}"); break; }
                };
                tokio::spawn(async move {
                    match TcpStream::connect(upstream).await {
                        Ok(mut target) => {
                            if let Err(e) = copy_bidirectional(&mut incoming, &mut target).await {
                                tracing::debug!("fnOS gateway connection ended: {e}");
                            }
                        }
                        Err(e) => tracing::warn!("fnOS gateway upstream unavailable: {e}"),
                    }
                });
            }
        });
        Ok(Some(Self { socket_path, socket_inode, worker }))
    }
}

impl Drop for GatewayProxy {
    fn drop(&mut self) {
        self.worker.abort();
        if let Ok(meta) = fs::symlink_metadata(&self.socket_path) {
            if meta.file_type().is_socket() && meta.ino() == self.socket_inode {
                let _ = fs::remove_file(&self.socket_path);
            }
        }
    }
}
