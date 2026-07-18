import nextVitals from "eslint-config-next/core-web-vitals";

const config = [
  {
    ignores: ["public/wasm/**", "src/lib/generated/**"]
  },
  ...nextVitals,
  {
    rules: {
      "react/no-unescaped-entities": "off"
    }
  }
];

export default config;
