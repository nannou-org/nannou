{ alsaLib
, cmake
, jq
, lib
, makeWrapper
, pkg-config
, rustPlatform
, vulkan-loader
, vulkan-validation-layers
, xorg
, openssl
, XCURSOR_THEME ? "Adwaita"
}:
rustPlatform.buildRustPackage rec {
  pname = "nannou";
  src = ./.;
  version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package.version;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "hotglsl-0.1.0" = "sha256-G88Sa/tgGppaxIIPXDqIazMWRBXpaSFb2mulNfCclm8=";
      "isf-0.1.0" = "sha256-utexaXpZZgpRunVAQyD2JAwvabhZGzeorC4pRFIumAc=";
      "skeptic-0.13.4" = "sha256-EZFtWIPfsfbpGBD8NwsVtMzRM10kVdg+djoV00dhT4Y=";
    };
  };

  # Don't run tests every time, we'll do it in a separate CI pass.
  doCheck = false;

  nativeBuildInputs = [
    # Required for `glsl-to-spirv` for `nannou_isf`. Should switch to `naga`.
    cmake
    makeWrapper
    pkg-config
  ];

  buildInputs = [
    # For filtering `cargo metadata` to get example names.
    jq
    # WGPU device availability.
    vulkan-loader
    vulkan-validation-layers
    # Required for X11 backend on Linux.
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    # `nannou-new` needs this because of `cargo` dep. See #606.
    openssl
    # `nannou_audio`.
    alsaLib
  ];

  env = {
    inherit XCURSOR_THEME;
    ALSA_LIB_DEV = "${alsaLib.dev}";
    LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
  };

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
  postFixup = ''
    for prog in $out/bin/* $out/bin/examples/*; do
      if [ -f "$prog" -a -x "$prog" ]; then
        wrapProgram "$prog" \
          --set ALSA_LIB_DEV "${env.ALSA_LIB_DEV}" \
          --set LD_LIBRARY_PATH "${env.LD_LIBRARY_PATH}" \
          --set XCURSOR_THEME "${env.XCURSOR_THEME}"
      fi
    done
  '';
}
