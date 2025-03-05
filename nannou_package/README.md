# nannou_package

> ## Deprecated
>
> This crate just created a zip file of the target artifacts and assets
> directory. See #1002.

A simple tool for packaging nannou project builds.

This tool is useful for packaging nannou projects into a named and dated
architecture-specific archive for distribution. The **nannou_package** tool does
the following:

1. Finds the parent Cargo.toml directory.
2. Finds the latest target/release/.
3. Creates a "builds" directory in the project root.
4. Creates "/name-arch-os-yyyymmdd-hhmmss/" inside "builds".
5. Copies the /target/release/ into the new directory.
6. Copies the assets directory into this new directory if it exists.
7. Zips the entire new directory.
8. Removes the new directory.

Install the **nannou_package** tool with the following:

```
cargo install nannou_package
```

Use the tool by changing to the project directory, running nannou_package and
following the prompts. **NOTE** that the project must be built before running
**nannou_package**, otherwise there will be no executable to package. In other
words one of the following two commands must be run before packaging:

- `cargo build --release`
- `cargo run --release`
