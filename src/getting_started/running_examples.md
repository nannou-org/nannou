# Running Examples

The easiest way to get familiar with Nannou is to explore the examples. To get
the examples we just need to clone the Nannou repository.

```bash
git clone https://github.com/nannou-org/nannou
```

If you do not have `git` installed you can press the "Clone or download" button
at the top of this page and then press "Download .zip".

Now, change the current directory to `nannou`.

```bash
cd nannou
```

Run the example using cargo.

```bash
cargo run --release --example simple_draw
```

The `--release` flag means we want to build with optimisations enabled.

The `--example` flag is a reference to the `[[examples]]` dictionary in the root
`Cargo.toml` file. Each key references a file in the `examples/` directory, so
to run the code at
`examples/nature_of_code/chp_01_vectors/1_1_bouncingball_novectors.rs` you would
run

```bash
cargo run --release --example 1_1_bouncingball_novectors
```

If you are compiling nannou for the first time you will see cargo download and
build all the necessary dependencies.

![cargo](https://i.imgur.com/5OBNqMB.gif)

Once the example compiles you should see the following window appear.

![simple_draw_HD](https://i.imgur.com/HVVamUI.gif)

To run any of the other examples, replace `simple_draw` with the name of the
desired example.
