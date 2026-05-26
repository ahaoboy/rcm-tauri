/**
 * Frontend type definitions for the RCM menu system.
 * Mirrors the Rust `rcm::Item` and `rcm::CommandPayload` structs.
 */

/** Serializable command payload sent to the Tauri `execute` command. */
export interface CommandPayload {
  exe: string;
  args?: string[];
  cwd?: string;
  admin?: boolean;
  window?: string;
}

/** A single item in the right-click menu (as received from the backend). */
export interface MenuItem {
  key: string;
  icon: string;
  label: string;
  disable: boolean;
  admin: boolean;
  window: string;
  items: MenuItem[];
  command: CommandPayload | null;
}

/** The full menu structure emitted by the backend. */
export interface MenuData {
  iconItems: MenuItem[];
  groups: MenuItem[];
}

/** Button type from input events. */
export type ButtonType = "Left" | "Right";

/** Payload received via the `input-event` Tauri event. */
export interface InputEventPayload {
  event: string;
  button: string;
  menu: MenuData;
}
