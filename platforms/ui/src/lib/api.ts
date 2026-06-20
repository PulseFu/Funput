// Typed wrappers over the Tauri commands exposed by the Windows shell
// (`platforms/windows/src-tauri/src/commands.rs`). Outside Tauri (e.g. `pnpm dev`
// in a plain browser) the calls no-op and reads return defaults, so the UI still
// renders for layout work.

import { invoke } from "@tauri-apps/api/core";

export type Method = "telex" | "vni";
export type Hotkey = "ctrl_backtick" | "ctrl_space" | "alt_shift";

export interface Settings {
  method: Method;
  enabled: boolean;
  smartRestore: boolean;
  eagerRestore: boolean;
  toggleHotkey: Hotkey;
  launchAtLogin: boolean;
  hasCompletedOnboarding: boolean;
}

const DEFAULTS: Settings = {
  method: "vni",
  enabled: true,
  smartRestore: true,
  eagerRestore: true,
  toggleHotkey: "ctrl_backtick",
  launchAtLogin: false,
  hasCompletedOnboarding: false,
};

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
  if (!inTauri) return undefined;
  return invoke<T>(cmd, args);
}

export async function getSettings(): Promise<Settings> {
  return (await call<Settings>("get_settings")) ?? structuredClone(DEFAULTS);
}

export const setMethod = (method: Method) => call("set_method", { method });
export const setEnabled = (on: boolean) => call("set_enabled", { on });
export const setSmartRestore = (on: boolean) => call("set_smart_restore", { on });
export const setEagerRestore = (on: boolean) => call("set_eager_restore", { on });
export const setToggleHotkey = (hotkey: Hotkey) => call("set_toggle_hotkey", { hotkey });
export const setLaunchAtLogin = (on: boolean) => call("set_launch_at_login", { on });
export const completeOnboarding = () => call("complete_onboarding");

export const HOTKEYS: { id: Hotkey; caps: string[] }[] = [
  { id: "ctrl_backtick", caps: ["Ctrl", "`"] },
  { id: "ctrl_space", caps: ["Ctrl", "Space"] },
  { id: "alt_shift", caps: ["Alt", "Shift"] },
];

export async function closeThisWindow(): Promise<void> {
  if (!inTauri) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().close();
}
