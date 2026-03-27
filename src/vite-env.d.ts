/// <reference types="vite/client" />

declare module "*.css";

// Tauri 全局类型扩展
interface Window {
  __TAURI_INTERNALS__?: unknown;
}
