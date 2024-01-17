{ lib
, nannou
, mkShell
, stdenv
}:
mkShell {
  name = "nannou-dev";
  inputsFrom = [ nannou ];
  env = (lib.optionalAttrs stdenv.isLinux
    {
      inherit (nannou) ALSA_LIB_DEV LD_LIBRARY_PATH XCURSOR_THEME;
    } // lib.optionalAttrs stdenv.isDarwin {
    inherit (nannou) COREAUDIO_SDK_PATH;
  });
}
