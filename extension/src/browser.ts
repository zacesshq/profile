// Firefox exposes the WebExtension API as browser and Chrome as chrome. Both return promises from
// the calls used here, so one name covers both
export const api: typeof chrome = (globalThis as unknown as { browser?: typeof chrome }).browser ?? chrome;

export const KEY = "zprf";

export interface Stored {
  /** The profile as parsed from TOML, already checked against the schema. */
  profile: import("./profile").Profile;
  /** The TOML it came from, shown again on the options page. */
  source: string;
  enabled: boolean;
}
