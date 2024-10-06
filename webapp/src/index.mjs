import init from "./pkg/graphics_toolbox.js";

export const runApp = () => {
  init().then(() => {
    console.log("WASM Loaded");
  }).catch(console.error);
};
