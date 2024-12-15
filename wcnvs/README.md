
## Instructions

1. Install npm: <https://docs.npmjs.com/downloading-and-installing-node-js-and-npm>
2. Install the tailwind css cli: <https://tailwindcss.com/docs/installation>
3. Run the following command in the root of the project to start the tailwind CSS compiler:

```bash
npm install tailwindcss -D #-g

npx tailwindcss -i tailwind_base.css -o assets/tailwind.css --watch
```

Launch the Dioxus Web app:

```bash
dx serve #--verbose
dx serve --platform web
```

Open the browser to <http://localhost:8080>

