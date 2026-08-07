// Cấu hình hệ thống Design Tokens Hoàng Gia cho TailwindCSS (Bảng màu Hoàng Kim, Đá Núi Lửa Obsidian, Sơn Son, Cẩm Thạch)
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,jsx,ts,tsx}"
  ],
  theme: {
    extend: {
      colors: {
        gold: {
          DEFAULT: '#D4AF37',
          dark: '#AA7C11',
          light: '#F3E5AB'
        },
        obsidian: {
          DEFAULT: '#0D1117',
          card: '#161B22',
          border: '#30363D'
        },
        vermilion: {
          DEFAULT: '#8B0000',
          glow: '#FF1A1A'
        },
        jade: {
          DEFAULT: '#004D40',
          light: '#00796B'
        }
      },
      fontFamily: {
        royal: ['Cinzel', 'serif'],
        body: ['Inter', 'sans-serif']
      },
      boxShadow: {
        glow: '0 0 15px rgba(212, 175, 55, 0.4), inset 0 0 15px rgba(212, 175, 55, 0.2)',
        glass: '0 8px 32px 0 rgba(0, 0, 0, 0.5)'
      }
    }
  },
  plugins: []
};
