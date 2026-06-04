# Nix Shell

The nannou repo offers a Nix shell as a way of providing a reproducible
environment across both macOS and Linux. This simplifies the nannou repo CI a
little while making it easier to experiment with nannou using a known working
environment.

### Installing Nix

The [Determinate Systems installer](https://github.com/DeterminateSystems/nix-installer)
is one of the easiest ways to get started with Nix.

### Nannou's Nix Files

- **default.nix** - A derivation that describes how to build the nannou Rust
  package, including all of its examples and with all of the necessary environment
  variables. This is what is built by default when running `nix build`. When
- **shell.nix** - A derivation that describes a development shell providing all
  of the necessary system dependencies and environment variables for working on
  nannou and its examples.
- **flake.nix** - A Nix manifest standard declaring the `nannou` package and
  devShell.

### Adding a `nannou` Dependency

When adding a new package (e.g. system dependency):

1. Find the package in nixpkgs (`nix search nixpkgs <name>` is handy)
2. Add the package to the `default.nix` input attribute set.
3. Add the package to `buildInputs` if its a runtime dependency, or
   `nativeBuildInputs` if its a build-time dependency.

### Adding a Development Shell Dependency

To add handy development dependencies (e.g. rustfmt, rust-analyser), do the same
but add them to `shell.nix` instead.

### Adding Environment Variables

Add these to the `env` attributes within either `default.nix` or `shell.nix`.

### Build Everything

Build all binaries, examples and dylibs within the `nannou` workspace with
`nix build`. When finished, look in the `result` symlink directory to find all
of the built binaries.

### Enter a nannou `devShell`

To enter a development shell with all of the tools and env vars necessary for
nannou dev, use `nix develop`.

To quickly enter a nannou dev shell to build a downstream nannou project, you
can use: `nix develop github:nannou-org/nannou`.
