import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "tests",
  // the extension test shares one browser profile, so the specs run one at a time
  workers: 1,
  timeout: 60_000,
  reporter: process.env.CI ? [["list"], ["github"]] : "list",
});
