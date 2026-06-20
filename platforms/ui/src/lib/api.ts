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

// Which OS shell hosts this UI. The shell appends `&platform=windows|linux` to the
// window URL (alongside `?view=`); empty in a plain browser (`pnpm dev`). Drives
// platform-specific styling and copy so one UI looks right on both shells.
export const PLATFORM =
  typeof window !== "undefined"
    ? (new URLSearchParams(location.search).get("platform") ?? "")
    : "";
export const isLinux = PLATFORM === "linux";

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

// Canonical Funput links, shared by every platform's About screen.
export const LINKS = {
  github: "https://github.com/pcodedynamics/Funput",
  website: "https://funput.pulsefu.com/",
};

/// Open a URL in the system browser. In Tauri this goes through the `open_url`
/// command (opener plugin); in a plain browser it falls back to `window.open`.
export async function openUrl(url: string): Promise<void> {
  if (!inTauri) {
    window.open(url, "_blank", "noopener");
    return;
  }
  await call("open_url", { url });
}

export async function closeThisWindow(): Promise<void> {
  if (!inTauri) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().close();
}

/// The app version (Tauri reads it from tauri.conf.json). "dev" outside Tauri.
export async function getAppVersion(): Promise<string> {
  if (!inTauri) return "dev";
  const { getVersion } = await import("@tauri-apps/api/app");
  return getVersion();
}
