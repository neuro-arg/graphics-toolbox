{ pkgs ? import <nixpkgs> { }, lib ? pkgs.lib }:

pkgs.mkShell rec {
  name = "shell-rust-wasm";

  # wasm requires https://github.com/oxalica/rust-overlay
  nativeBuildInputs = with pkgs; if pkgs?rust-bin then [
    (rust-bin.stable.latest.default.override {
      targets = [ "wasm32-unknown-unknown" ];
    })
    wasm-pack
    wasm-bindgen-cli
    binaryen
  ] else [
    rustc
    cargo
  ];

  buildInputs = with pkgs; [
    # openssl
    # alsa-lib
    # xorg.libX11
    # xorg.libXcursor
    # xorg.libXrandr
    # xorg.libXi
    wayland
    libxkbcommon
    # libGL
    vulkan-loader
    yarn
    nodejs
  ];

  LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
}
