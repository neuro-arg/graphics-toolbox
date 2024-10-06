const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  entry: "./src/index.js",
  output: {
    path: dist,
    filename: "[name].js"
  },
  plugins: [
    new CopyPlugin({ patterns: ['./static/index.html'] }),

    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, ".."),
      outName: 'graphics_toolbox',
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
