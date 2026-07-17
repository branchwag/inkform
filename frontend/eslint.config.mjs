import nextVitals from "eslint-config-next/core-web-vitals";

const config = [
  {
    ignores: ["public/wasm/**"]
  },
  ...nextVitals,
  {
    rules: {
      "react/no-unescaped-entities": "off"
    }
  }
];

export default config;
