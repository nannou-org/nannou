# Publishing New Versions

The nannou repo allows community members to open a PR for publishing a new
version of any or all of the nannou crates. This makes it much easier for
maintainers to publish a new release - all we have to do is accept your PR!

If you would like to see a new version of a crate published, follow these steps:

## Choose a Version

The version to use for the next version of the crate(s) you wish to publish will
depend on the changes that have been made to the crate(s). The nannou crates
follow the rust convention which you can read about [here][cargo-toml-version].

> _**Hot tip**! Each of the numbers in a crate version has its own name:_
>
> ```
> MAJOR.MINOR.PATCH
> ```
>
> _E.g. in the version `0.15.3` the `0` is the "major" version, the `15` is the
> "minor" version and the `3` is the "patch" or "tiny" version. Keep an eye out
> for these names when reading about versioning in Rust._

Also necessary to keep in mind is that nannou synchronises versions that
represent a breaking change (e.g. a change from `0.15.3` to `0.16.0` or `1.0.4`
to `2.0.0`). In these cases, all crates with the name `nannou` or `nannou_*`
should be published together with the same version. This version synchronisation
makes it easier for users to intuit compatible versions of nannou crates without
the need to manually check all of the dependency versions on crates.io.

## Update Cargo.toml

There are two sections of the `Cargo.toml` file(s) that will need updating.

1. The `version` field under the `[package]` section.
2. The `version` field of the `[dependencies]` and `[dev-dependencies]`
   sections of each crate in the repo that uses the crate. E.g. the `nannou`
   crate is a dependency of `nannou_isf`. If we wish to update the version of
   `nannou`, we will also need to update the version of `nannou` specified in
   the `[dependencies]` section of `nannou_isf`.

This can be quite a lot of Cargo.toml changes in the case that you are updating
the version of all of the `nannou_*` crates!

To make this easier, the nannou repo includes a small program at
`scripts/set_version`. You can use it like so:

```
cargo run --bin set_version -- "0.42.0"
```

This will:

- Find all crates via the cargo workspace Cargo.toml file.
- Sets the specified version number for each of the `nannou*` packages and
  updates each of their respective `nannou*` dependencies.
- Edits their Cargo.toml files with the result.

## Update the Guide

There are two places where we must update the version in the guide:

1. The **Changelog**. You can find it at `guide/src/changelog.md`. See the most
   recent version in the guide for a demonstration of how to update the version.
   For the most part, this just involves adding a date and release heading under
   the `Unreleased` heading.
2. Update the nannou version number in step 3 of the
   `guide/src/getting_started/create_a_project.md` section.  See the
   `[dependencies]` section of the code snippet.

Otherwise, we avoid referring to specific versions in the guide to make updating
easier. If you happen to be familiar with grep, this command can help you to
double check that there are no more places to update:

```
grep -nr "= \"0.14\"" guide/
```

where `0.14` would be replaced with the beginning of the previous version of
`nannou`. This should should list the files and line numbers where the previous
version still exists and likely needs updating.

## Open a PR

Now you should be ready to open a PR! Be sure to follow the [PR
Checklist](./pr-checklist.md).

Once your PR is reviewed and merged, the nannou repo's CI bot will automatically
publish the new versions.

Congrats, you just published some new goodies to crates.io for the nannou
community!

[cargo-toml-version]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-version-field
