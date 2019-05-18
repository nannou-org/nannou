# The Nannou Guide

The one-stop-shop for everything someone might want to know about Nannou!

## Working on the Book

The easiest way to build, render and read the book is as follows:

- Clone the repo.
  ```bash
  git clone https://github.com/nannou-org/guide
  cd guide
  ```
- Install the Rust markdown book tool.
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
