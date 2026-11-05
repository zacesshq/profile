// opens the options page on first install, since nothing happens until a profile is loaded, and
// whenever the toolbar button is pressed

import { api, KEY } from "./browser";

api.runtime.onInstalled.addListener((details) => {
  if (details.reason !== "install") return;
  void api.storage.local.get(KEY).then((items) => {
    if (!items[KEY]) void api.runtime.openOptionsPage();
  });
});

api.action.onClicked.addListener(() => {
  void api.runtime.openOptionsPage();
});
