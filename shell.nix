{ lib
, mdbook
, mdbook-linkcheck2
, nannou
, mkShell
, rust-analyzer
, rustfmt
, stdenv
}:
mkShell {
  name = "nannou-dev";
  inputsFrom = [ nannou ];
  buildInputs = [
    mdbook
    mdbook-linkcheck2
    rust-analyzer
    rustfmt
  ];
  env = (lib.optionalAttrs stdenv.isLinux
    {
      inherit (nannou) ALSA_LIB_DEV LD_LIBRARY_PATH XCURSOR_THEME;
    });
}
