{ nannou
, mkShell
}:
mkShell {
  name = "nannou-dev";
  inputsFrom = [ nannou ];
  env = {
    inherit (nannou) ALSA_LIB_DEV LD_LIBRARY_PATH XCURSOR_THEME;
  };
}
