# Publishing New Versions

The nannou repo allows community members to open a PR for publishing a new
version of the nannou crates. This makes it much easier for maintainers to
publish a new release - all we have to do is accept your PR!

Publishing is automated with [`release-plz`][release-plz]. Once a version-bump
PR is merged into `master`, CI publishes every nannou crate whose version isn't
yet on crates.io - in the correct dependency order - then creates the `vX.Y.Z`
git tag and a GitHub Release with auto-generated notes. You never need to run
`cargo publish` by hand.

> _**Maintainers**: publishing authenticates to crates.io with [Trusted
> Publishing][trusted-publishing] (no stored API token). See [Trusted Publishing
> setup](#trusted-publishing-setup) below for the one-time configuration._

If you would like to see a new version published, follow these steps.

## Choose a Version

The version to use depends on the changes that have been made. The nannou crates
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

By convention the published `nannou` crates share a single version (the
workspace version) and are bumped together, so users can reason about compatible
versions at a glance. This is a convention rather than a hard requirement -
`set_version` (below) bumps them together for you, but nothing stops you choosing
versions differently when a release calls for it. Crates that aren't published -
the examples and various support crates - keep their own versions and are left
alone.

## Bump the Version

The version lives in a single place - `[workspace.package].version` in the root
`Cargo.toml` - which the published crates inherit via `version.workspace = true`.
The matching version requirements for the internal `nannou_*` dependencies live
in the root `[workspace.dependencies]` table.

To bump both at once, use the small program at `scripts/set_version`:

```
cargo run --bin set_version -- "0.42.0"
```

This sets `[workspace.package].version` and the `version` of each internal
`nannou*` entry in `[workspace.dependencies]` to the new version.

## Update the Guide

There are two places to update in the guide:

1. The **Changelog** at `guide/src/changelog.md`. This is hand-written prose,
   maintained separately from the auto-generated GitHub Release notes - it's
   where we describe a release's highlights and any migration notes. Add a date
   and release heading under the `Unreleased` heading (see the most recent
   version for an example).
2. The nannou version number in step 3 of
   `guide/src/getting_started/create_a_project.md` (the `[dependencies]` section
   of the code snippet).

Otherwise we avoid referring to specific versions in the guide to make updating
easier. If you're familiar with grep, this can help double-check there are no
more places to update:

```
grep -nr "= \"0.14\"" guide/
```

where `0.14` is the beginning of the previous version of `nannou`. This lists
the files and line numbers where the previous version still appears and likely
needs updating.

## Open a PR

Now you should be ready to open a PR! Be sure to follow the [PR
Checklist](./pr-checklist.md).

> _**Tip**: writing [Conventional Commit][conventional-commits] messages (`feat:`,
> `fix:`, `feat!:` for breaking changes, etc.) keeps the auto-generated GitHub
> Release notes tidy, since `release-plz` groups them by type._

Once your PR is reviewed and merged, CI publishes the new versions, tags the
release and cuts a GitHub Release automatically.

Congrats, you just published some new goodies to crates.io for the nannou
community!

## Trusted Publishing setup

> _This section is for maintainers - it's a one-time setup, not part of the
> per-release flow above._

CI authenticates to crates.io with [Trusted Publishing][trusted-publishing] over
GitHub OIDC, so there is no long-lived crates.io API token stored as a secret.
The release job mints a short-lived token at publish time via
[`rust-lang/crates-io-auth-action`][auth-action] (this is why the job has the
`id-token: write` permission).

For this to work, **each published crate needs a trusted publisher configured on
crates.io**. On the crate's crates.io page, go to *Settings -> Trusted
Publishing -> Add a new trusted publisher* and enter:

- **Repository owner**: `nannou-org`
- **Repository name**: `nannou`
- **Workflow filename**: `ci.yml`
- **Environment**: leave empty (or set a GitHub Actions environment name if you
  also add a matching `environment:` to the release job for extra protection)

crates.io only lets you add a trusted publisher for a crate that **already
exists**. Any crate that has never been published must therefore be published
once with a token first (e.g. a one-off `cargo publish` using a personal API
token), after which its trusted publisher can be configured. From then on every
release goes through CI with no stored secret.

[cargo-toml-version]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-version-field
[conventional-commits]: https://www.conventionalcommits.org
[release-plz]: https://release-plz.dev
[trusted-publishing]: https://crates.io/docs/trusted-publishing
[auth-action]: https://github.com/rust-lang/crates-io-auth-action
