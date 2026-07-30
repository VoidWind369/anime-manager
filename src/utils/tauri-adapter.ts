import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import type { Anime, Episode, ScanResult } from "../types/anime";

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return tauriInvoke(cmd, args) as Promise<T>;
}