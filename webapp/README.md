## How to install

```sh
npm install
```

## How to run in debug mode

```sh
# Builds the project and opens it in a new browser tab. Auto-reloads when the project changes.
npm start
```

## How to build in release mode

```sh
# Builds the project and places it into the `dist` folder.
npm run build
```

## How to run unit tests

```sh
# Runs tests in Firefox
npm test -- --firefox

# Runs tests in Chrome
npm test -- --chrome

# Runs tests in Safari
npm test -- --safari
```

## What does each file do?

* `package.json` contains the standard npm metadata. You put your JavaScript dependencies in here. You must change this file with your details (author, name, version)

* The `src` folder contains your JavaScript code (`index.js` is used to hook everything into Webpack, you don't need to change it).

* The `static` folder contains any files that you want copied as-is into the final build. It contains an `index.html` file which loads the `index.js` file.
