{ alsa-lib
, ffmpeg
, jq
, lib
, libxkbcommon
, llvmPackages
, makeWrapper
, openssl
, pkg-config
, rustPlatform
, vulkan-loader
, vulkan-validation-layers
, xorg
, stdenv
, udev
, XCURSOR_THEME ? "Adwaita"
}:
rustPlatform.buildRustPackage rec {
  pname = "nannou";
  src = lib.sourceFilesBySuffices ./. [
    ".rs"
    ".toml"
    ".lock"
    ".wgsl"
  ];
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
    llvmPackages.clang
    pkg-config
  ];

  buildInputs = ([
    ffmpeg
    # For filtering `cargo metadata` to get example names.
    jq
    # Needed by `reqwest`, used in `generative_design` wikipedia example.
    openssl
  ] ++ lib.optionals stdenv.isLinux [
    alsa-lib
    udev
    libxkbcommon
    llvmPackages.bintools
    llvmPackages.libclang
    vulkan-loader
    vulkan-validation-layers
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ] ++ lib.optionals stdenv.isDarwin [
    rustPlatform.bindgenHook
  ]);

  env = (lib.optionalAttrs stdenv.isLinux
    {
      inherit XCURSOR_THEME;
      LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
      ALSA_LIB_DEV = "${alsa-lib.dev}";
      LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
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
    in
    ''
      for prog in $out/bin/* $out/bin/examples/*; do
        if [ -f "$prog" -a -x "$prog" ]; then
          wrapProgram "$prog" \
            ${linuxWrapArgs}
        fi
      done
    '';
}
