/** fnOS HTTPS 内嵌页面使用同源 API/SSE；独立入口保留旧路径。 */
export const FNOS_GATEWAY_PREFIX = "/app/motrix";
export function resolveApiPath(path: string, pathname = currentPathname()): string {
  const inGateway = pathname === FNOS_GATEWAY_PREFIX || pathname.startsWith(FNOS_GATEWAY_PREFIX + "/");
  return inGateway && path.startsWith("/") ? FNOS_GATEWAY_PREFIX + path : path;
}
function currentPathname(): string {
  return typeof window === "undefined" ? "/" : window.location.pathname;
}
