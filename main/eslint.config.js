import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import tailwind from "eslint-plugin-tailwindcss";

export default [
  // eslint-plugin-react 尚未支持 ESLint 10，暂不启用；
  // react-hooks 7 已覆盖 hooks 规则，jsx-runtime 模式无需手动引入 React。
  {
    ...tailwind.configs.recommended,
    settings: {
      ...tailwind.configs.recommended.settings,
      tailwindcss: { cssConfigPath: "src/index.css" },
    },
  },
  { ignores: ["dist", "src/subapps/journal/src/**"] },
  {
    files: ["**/*.test.{js,cjs}"],
    languageOptions: { globals: globals.node },
  },
  {
    files: ["**/*.{js,jsx}"],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
      parserOptions: {
        ecmaVersion: "latest",
        ecmaFeatures: { jsx: true },
        sourceType: "module",
      },
    },
    settings: {
      react: { version: "19.3" },
      tailwindcss: { cssConfigPath: "src/index.css" },
    },
    plugins: {
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    rules: {
      ...js.configs.recommended.rules,
      ...reactHooks.configs.recommended.rules,
      "react-refresh/only-export-components": [
        "warn",
        { allowConstantExport: true },
      ],
      // 新版插件的严格规则先以警告提示，不阻断现有代码
      "tailwindcss/no-contradicting-classname": "warn",
      "react-hooks/set-state-in-effect": "warn",
      "react-hooks/static-components": "warn",
      "react-hooks/refs": "warn",
      "react-hooks/purity": "warn",
      "react-hooks/immutability": "warn",
      "react-hooks/preserve-manual-memoization": "warn",
    },
  },
];
