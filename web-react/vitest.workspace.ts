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
]);