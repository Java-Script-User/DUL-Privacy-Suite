/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: '#2563eb',
        success: '#10b981',
        danger: '#ef4444',
        warning: '#f59e0b',
        dark: {
          900: '#0f172a',
          800: '#1e293b',
          700: '#334155',
        }
      }
    },
  },
  plugins: [],
}
