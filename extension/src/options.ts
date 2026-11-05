import { api, KEY, type Stored } from "./browser";
import { buildCss } from "./css";
import { parseProfile } from "./profile";

const $ = <T extends HTMLElement>(id: string): T => {
  const el = document.getElementById(id);
  if (!el) throw new Error(`the options page has no #${id}`);
  return el as T;
};

const source = $<HTMLTextAreaElement>("source");
const file = $<HTMLInputElement>("file");
const status = $<HTMLParagraphElement>("status");
const errors = $<HTMLUListElement>("errors");
const current = $<HTMLElement>("current");
const enabled = $<HTMLInputElement>("enabled");
const access = $<HTMLElement>("access");

const list = (ul: HTMLElement, items: string[]): void => {
  ul.replaceChildren(
    ...items.map((text) => {
      const li = document.createElement("li");
      li.textContent = text;
      return li;
    }),
  );
};

async function show(): Promise<void> {
  const stored = (await api.storage.local.get(KEY))[KEY] as Stored | undefined;
  if (!stored) {
    current.hidden = true;
    return;
  }
  current.hidden = false;
  if (!source.value) source.value = stored.source;
  $("current-name").textContent = stored.profile.name;
  enabled.checked = stored.enabled;
  const built = buildCss(stored.profile);
  list($("applied"), built.applied);
  list($("skipped"), built.skipped);
  $("skipped-heading").hidden = built.skipped.length === 0;
}

async function load(text: string): Promise<void> {
  errors.replaceChildren();
  const result = parseProfile(text);
  if (!result.ok) {
    status.textContent = "This profile was not loaded. Fix the problems listed below and try again.";
    list(errors, result.errors);
    errors.focus();
    return;
  }
  const stored: Stored = { profile: result.profile, source: text, enabled: true };
  await api.storage.local.set({ [KEY]: stored });
  status.textContent = `Loaded ${result.profile.name}. Every open page now uses it.`;
  await show();
}

$("load").addEventListener("click", () => void load(source.value));

file.addEventListener("change", () => {
  const picked = file.files?.[0];
  if (!picked) return;
  void picked.text().then((text) => {
    source.value = text;
    return load(text);
  });
});

enabled.addEventListener("change", async () => {
  const stored = (await api.storage.local.get(KEY))[KEY] as Stored | undefined;
  if (!stored) return;
  await api.storage.local.set({ [KEY]: { ...stored, enabled: enabled.checked } });
  status.textContent = enabled.checked ? "The profile is on for every page." : "The profile is off. Pages show their own styles.";
});

$("remove").addEventListener("click", async () => {
  await api.storage.local.remove(KEY);
  source.value = "";
  status.textContent = "The profile was removed. Pages show their own styles.";
  await show();
});

// Firefox treats site access as optional, so the person may need to grant it once
const ALL = { origins: ["<all_urls>"] };
async function checkAccess(): Promise<void> {
  access.hidden = await api.permissions.contains(ALL);
}
$("grant").addEventListener("click", async () => {
  await api.permissions.request(ALL);
  await checkAccess();
});

void show();
void checkAccess();
