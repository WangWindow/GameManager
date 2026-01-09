import { invoke } from "@tauri-apps/api/core";

export type InvokeResult<T> = { ok: true; data: T } | { ok: false; error: string };

function formatInvokeError(err: unknown): string {
  if (typeof err === "string") return err;
  if (err instanceof Error) return err.message;

  if (err && typeof err === "object") {
    const anyErr = err as Record<string, unknown>;
    const message = anyErr.message;
    if (typeof message === "string" && message.trim().length > 0) return message;
  }

  try {
    return JSON.stringify(err);
  } catch {
    return String(err);
  }
}

export async function invokeResult<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<InvokeResult<T>> {
  try {
    const data = await invoke<T>(command, args);
    return { ok: true, data };
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error(`[tauri] invoke failed: ${command}`, err);
    return { ok: false, error: formatInvokeError(err) };
  }
}

export async function safeInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    return await invoke<T>(command, args);
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error(`[tauri] invoke failed: ${command}`, err);
    return null;
  }
}
