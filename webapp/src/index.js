import init from "web";

export const runApp = async () => {
  try {
    await init();
    console.log("WASM Loaded");
  } catch (e) {
    console.error(e);
  }
};

runApp();
