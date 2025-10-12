import forms from '@tailwindcss/forms'
import containerQueries from '@tailwindcss/container-queries'
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './index.html',
    './src/**/*.{vue,ts,tsx,js,jsx}'
  ],
  theme: {
    extend: {},
  },
  plugins: [
    forms,
    containerQueries,
  ]
}
