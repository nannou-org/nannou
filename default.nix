{ alsaLib
, darwin
, ffmpeg
, jq
, lib
, llvmPackages
, libiconv
, makeWrapper
, pkg-config
, rustPlatform
, vulkan-loader
, vulkan-validation-layers
, xorg
, openssl
, stdenv
, udev
, XCURSOR_THEME ? "Adwaita"
}:
rustPlatform.buildRustPackage rec {
  pname = "nannou";
  src = ./.;
  version = (builtins.fromTOML (builtins.readFile ./nannou/Cargo.toml)).package.version;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "skeptic-0.13.8" = "sha256-LLVrpuyQsMdbp8OYcHN0nq+uKC8xgJzpNy+gyXxTYbo=";
    };
  };

  # Don't run tests every time, we'll do it in a separate CI pass.
  doCheck = false;

  nativeBuildInputs = [
    makeWrapper
    pkg-config
  ];

  buildInputs = ([
    # For filtering `cargo metadata` to get example names.
    jq
    # `nannou-new` needs this because of `cargo` dep. See #606.
    openssl
    ffmpeg
  ] ++ lib.optionals stdenv.isLinux [
    alsaLib
    udev
    llvmPackages.bintools
    vulkan-loader
    vulkan-validation-layers
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.AppKit
    darwin.apple_sdk.frameworks.AudioToolbox
    darwin.apple_sdk.frameworks.AudioUnit
    darwin.apple_sdk.frameworks.CoreAudio
    libiconv
    rustPlatform.bindgenHook
  ]);

  env = (lib.optionalAttrs stdenv.isLinux
    {
      inherit XCURSOR_THEME;
      LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
      ALSA_LIB_DEV = "${alsaLib.dev}";
    } // lib.optionalAttrs stdenv.isDarwin {
    COREAUDIO_SDK_PATH = "${darwin.apple_sdk.frameworks.CoreAudio}/Library/Frameworks/CoreAudio.framework";
  });


  # Build and include example binaries in `$out/bin/examples`
  postBuild = ''
    cargo build --locked --release --examples
    mkdir -p $out/bin/examples
    for example in $(cargo metadata --format-version=1 --no-deps | ${jq}/bin/jq -r '.packages[].targets[] | select(.kind[] | contains("example")) | .name'); do
      if [ -f "target/release/examples/$example" ]; then
        mv "target/release/examples/$example" $out/bin/examples/
      fi
    done
  '';

  # Wrap the binaries to ensure the runtime env vars are set.
  postFixup =
    let
      linuxWrapArgs = lib.optionalString stdenv.isLinux ''\
      --set LD_LIBRARY_PATH "${env.LD_LIBRARY_PATH}" \
      --set ALSA_LIB_DEV "${env.ALSA_LIB_DEV}" \
      --set XCURSOR_THEME "${env.XCURSOR_THEME}"'';
      macosWrapArgs = lib.optionalString stdenv.isDarwin ''\
      --set COREAUDIO_SDK_PATH "${env.COREAUDIO_SDK_PATH}"'';
    in
    ''
      for prog in $out/bin/* $out/bin/examples/*; do
        if [ -f "$prog" -a -x "$prog" ]; then
          wrapProgram "$prog" \
            ${linuxWrapArgs} \
            ${macosWrapArgs}
        fi
      done
    '';
}
