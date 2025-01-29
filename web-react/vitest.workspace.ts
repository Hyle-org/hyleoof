import { defineWorkspace } from "vitest/config";

export default defineWorkspace([
  {
    extends: "./vite.config.ts",
    test: {
      include: ["**/*.node.test.{ts,tsx}"],
      name: "happy-dom",
      environment: "happy-dom",
    },
  },
  {
    extends: "./vite.config.js",
    test: {
      setupFiles: ["vitest-browser-react"],
      include: ["**/*.browser.test.{ts,tsx}"],
      name: "browser",
      browser: {
        provider: "playwright",
        enabled: true,
        name: "firefox",
      },
    },
  },
]);