import { start } from "web";

export const runApp = () => {
  try {
    start();
    console.log("WASM Loaded");
  } catch (e) {
    console.error(e);
  }
};

runApp();
