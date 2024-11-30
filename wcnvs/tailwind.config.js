/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{rs,html,css}", "./dist/**/*.html"],
  //mode: "all",

  theme: {
    //container: { center: true, },
    extend: { },
  },

  plugins: [
    //require('@tailwindcss/forms'),
    //require('tailwindcss-children'),
    //require('@tailwindcss/typography'),
    //require('@tailwindcss/aspect-ratio'),
    //require('@tailwindcss/line-clamp'),
    //require('tw-elements/plugin.cjs'), // npm install tw-elements
  ],

  //presets: [ require('@acmecorp/tailwind-base') ],
  // https://github.com/tailwindlabs/tailwindcss/blob/master/stubs/defaultConfig.stub.js
  // npm install -D tailwindcss // npx tailwindcss init #--full
}
