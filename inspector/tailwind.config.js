/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        ink: '#0B0E14',
        panel: '#12161F',
        panel2: '#171C27',
        rule: '#232A38',
        ruleSoft: '#1A2029',
        signal: '#4FD3C4',
        signalDim: '#2B7A70',
        seal: '#C1392B',
        brass: '#C89B3C',
        paper: '#E8E4D9',
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'monospace'],
        serif: ['Lora', 'serif'],
        sans: ['Outfit', 'sans-serif'],
      }
    },
  },
  plugins: [],
}
