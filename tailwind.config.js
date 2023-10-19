/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{jinja,html,rs}",],
  theme: {
    extend: {},
  },
  plugins: [
    require("@tailwindcss/typography"),
    require("daisyui"),
  ],
  daisyui: {
    themes: ["light", "dark", "business"],
  },
}

