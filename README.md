# the nannou guide [![Build Status](https://travis-ci.org/nannou-org/guide.svg?branch=master)](https://travis-ci.org/nannou-org/guide)

The one-stop-shop for everything someone might want to know about nannou!

## Working on the Book

The easiest way to build, render and read the book is as follows:

- Clone the repo.
  ```bash
  git clone https://github.com/nannou-org/guide
  cd guide
  ```
- Install the [Rust markdown book](https://github.com/rust-lang-nursery/mdBook) tool.
  ```
  cargo install mdbook
  ```
- Make `mdbook` watch the repo, re-build on file changes and host at
  `localhost:3000`.
  ```
  mdbook serve
  ```
- Open your browser and point it to `localhost:3000` to find the rendered
  markdown book.

You should now have a hot-loading environment where you can edit the book
markdown and see the results rendered in your browser each time you save a file.

## Running Tests

To run the tests, do the following:

```bash
cd book-tests
cargo test
```

The `build.rs` will retrieve all `rust` code snippets from the markdown files
and generate a test file so that they all may be tested during `cargo test`.

We do this rather than using the `mdbook test` as `mdbook test` does not support
including remote dependencies.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

**Contributions**

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
