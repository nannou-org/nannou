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
      perSystemPkgs = f: perSystem (system: f system (systemPkgs system));
    in
    {
      packages = perSystemPkgs (system: pkgs: {
        nannou = pkgs.callPackage ./default.nix { };
        default = inputs.self.packages.${system}.nannou;
      });

      apps = perSystemPkgs (system: pkgs:
        let
          nannou = inputs.self.packages.${system}.nannou;
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

      devShells = perSystemPkgs (system: pkgs: {
        nannou-dev = pkgs.callPackage ./shell.nix {
          inherit (inputs.self.packages.${system}) nannou;
        };
        default = inputs.self.devShells.${system}.nannou-dev;
      });

      formatter = perSystemPkgs (_: pkgs: pkgs.nixpkgs-fmt);
    };
}
