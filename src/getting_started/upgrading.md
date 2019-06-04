# Upgrading nannou

You can upgrade to a new version of nannou by downloading the release from [github](https://github.com/nannou-org/nannou).
To do that run either
```bash
git clone https://github.com/nannou-org/nannou.git
```
or download the repo as an archive.

Finally run
```bash
cargo update
```
inside the nannou directory to upgrade all dependencies.

Building Nannou examples might still fail. This is most likely due to new language features.
Nannou 0.9 for example requires rustc 1.35.0.
You can upgrade you (local) rust toolchain by executing
```bash
rustup update
```
