{
  description = ''
    A Nix flake for nannou development.
  '';

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
  };

  outputs = inputs:
    let
      systems = import inputs.systems;
      lib = inputs.nixpkgs.lib;
      perSystem = f: lib.genAttrs systems f;
      systemPkgs = system: import inputs.nixpkgs { inherit system; };
      perSystemPkgs = f: perSystem (system: f (systemPkgs system));
    in
    {
      packages = perSystemPkgs (pkgs: {
        nannou = pkgs.callPackage ./default.nix { };
        default = inputs.self.packages.${pkgs.system}.nannou;
      });

      apps = perSystemPkgs (pkgs:
        let
          nannou = inputs.self.packages.${pkgs.system}.nannou;
        in
        {
          draw = {
            type = "app";
            program = "${nannou}/bin/examples/draw";
          };
          draw_textured_mesh = {
            type = "app";
            program = "${nannou}/bin/examples/draw_textured_mesh";
          };
        });

      devShells = perSystemPkgs (pkgs: {
        nannou-dev = pkgs.callPackage ./shell.nix {
          inherit (inputs.self.packages.${pkgs.system}) nannou;
        };
        default = inputs.self.packages.${pkgs.system}.nannou;
      });

      formatter = perSystemPkgs (pkgs: pkgs.nixpkgs-fmt);
    };
}
