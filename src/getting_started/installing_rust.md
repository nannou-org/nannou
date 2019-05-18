# Installing Rust

Nannou is a library written for the [Rust programming
language](https://www.rust-lang.org/). Thus, the first step is to install Rust!

To install Rust, open up your terminal, copy the text below, paste it into your
terminal and hit enter.

```bash
curl https://sh.rustup.rs -sSf | sh
```

Now Rust is installed!

Next we will install some tools that help IDEs do fancy things like
auto-completion and go-to-definition.

```bash
rustup component add rust-src rustfmt-preview rust-analysis
```

Please see [this link](https://www.rust-lang.org/tools/install) if you would
like more information on the Rust installation process.
