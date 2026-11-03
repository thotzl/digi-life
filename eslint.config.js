// ESLint flat config for the frontend (TCK-161): typescript-eslint recommended plus two adjustments; `npm run lint`
// fails on any warning.
import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  { ignores: ["dist/", "node_modules/", "target/", "bindings/", "gen/", ".agents/"] },
  {
    files: ["frontend/**/*.ts"],
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    rules: {
      // Same convention as tsc's noUnusedParameters: a leading underscore marks an intentionally unused name
      "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_", varsIgnorePattern: "^_" }],
    },
  },
  {
    // Tests build partial fixtures and fake globals (window, IPC) that are not worth typing in full
    files: ["frontend/**/*.test.ts"],
    rules: { "@typescript-eslint/no-explicit-any": "off" },
  },
);
