/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    fontFamily: {
      sans: ["Space Grotesk", "sans-serif"],
      serif: ["PT Serif", "serif"],
    },
    fontSize: {
      xs: "0.75rem",
      sm: "0.875rem",
      base: "0.875rem",
      "base-sans": "1rem",
      lg: "1.125rem",
      xl: "1.5rem",
      "2xl": "2rem",
    },
    borderRadius: {
      base: "0.5rem",
      sm: "0.25rem",
      xs: "0.156rem",
      lg: "1rem",
      xl: "2rem",
      max: "999999px",
    },
    extend: {
      colors: {
        "brand-sea": {
          100: "#C5ECE0",
          200: "#17C3B2",
          300: "#1DA0A8",
          400: "#227C9D",
          500: "#094074",
        },
        "brand-sunset": {
          100: "#FEF9EF",
          200: "#FFEED1",
          300: "#FFE2B3",
          400: "#FFD795",
          500: "#FFCB77",
        },
        "brand-red": {
          100: "#FED6D0",
          200: "#FEB3B1",
          300: "#FE9092",
          400: "#FE7F83",
          500: "#FE6D73",
        },
        "brand-black": "#161a1e",
        "brand-white": "#fbfbfb",
      },
    },
  },
  plugins: [],
};
