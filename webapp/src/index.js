import init from "web";

export const runApp = () => {
  try {
    init();
    console.log("WASM Loaded");
  } catch (e) {
    console.error(e);
  }
};

runApp();
