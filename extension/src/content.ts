// runs at document_start in every frame and keeps one style element in step with the stored
// profile, so a change on the options page reaches every open tab without a reload

import { api, KEY, type Stored } from "./browser";
import { buildCss } from "./css";

const ID = "zacess-profile";

function apply(stored: Stored | undefined): void {
  document.getElementById(ID)?.remove();
  if (!stored?.enabled || !stored.profile) return;
  const style = document.createElement("style");
  style.id = ID;
  style.textContent = buildCss(stored.profile).css;
  // the head may not exist yet at document_start; the root element always does
  (document.head ?? document.documentElement).append(style);
}

void api.storage.local.get(KEY).then((items) => apply(items[KEY] as Stored | undefined));

api.storage.onChanged.addListener((changes, area) => {
  if (area === "local" && KEY in changes) apply(changes[KEY]?.newValue as Stored | undefined);
});
