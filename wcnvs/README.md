
## Instructions

1. Install npm: <https://docs.npmjs.com/downloading-and-installing-node-js-and-npm>
2. Install the tailwind css cli: <https://tailwindcss.com/docs/installation>
3. Run the following command in the root of the project to start the tailwind CSS compiler:

```bash
npm install tailwindcss @tailwindcss/cli #-D -g

npx tailwindcss -i input.css -o assets/tailwind.css -w #-m
```

Launch the Dioxus Web app:

```bash
dx serve --web #--verbose
```

Open the browser to <http://localhost:8080>

