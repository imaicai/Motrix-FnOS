import { describe, expect, it } from "vitest";
import { resolveApiPath } from "./apiBase";
describe("resolveApiPath", () => {
  it("fnOS HTTPS 嵌入网关地址", () => {
    expect(resolveApiPath("/api/app/ping", "/app/motrix/")).toBe("/app/motrix/api/app/ping");
    expect(resolveApiPath("/api/events", "/app/motrix/")).toBe("/app/motrix/api/events");
  });
  it("独立端口与相似路径不会误加前缀", () => {
    expect(resolveApiPath("/api/app/ping", "/")).toBe("/api/app/ping");
    expect(resolveApiPath("/api/auth/status", "/app/motrix-copy/")).toBe("/api/auth/status");
  });
});
