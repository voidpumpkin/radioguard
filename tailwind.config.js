/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{jinja,html}"],
  theme: {
    extend: {},
  },
  plugins: [
    require("daisyui"),
  ],
  daisyui: {
    themes: ["light", "dark", "business"],
  },
}

