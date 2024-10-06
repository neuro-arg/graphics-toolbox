import path from "node:path";
import CopyPlugin from "copy-webpack-plugin";
import WasmPackPlugin from "@wasm-tool/wasm-pack-plugin";

const dist = path.resolve(import.meta.dirname, "dist");

export default {
  mode: "production",
  devtool: "inline-source-map",
  entry: "./src/index.mjs",
  output: {
    path: dist,
    filename: "[name].js"
  },
  plugins: [
    new CopyPlugin({ patterns: ['./static/index.html'] }),

    new WasmPackPlugin({
      crateDirectory: path.resolve(import.meta.dirname, ".."),
      outName: 'graphics_toolbox',
      extraArgs: '--target web',
    }),
  ],
  experiments: {
    asyncWebAssembly: true
  },
  performance: {
    hints: false,
    maxEntrypointSize: 512000,
    maxAssetSize: 512000
  }
};
