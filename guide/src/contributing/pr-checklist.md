# PR Checklist

When creating a PR, there are a few things you can do to help the process of
landing your changes go smoothly:

- **Code Formatting** - The most common step to forget is running `cargo fmt
  --all` (I forget all the time)! This step ensures there is a consistent style
  across nannou's codebase.
- **Errors** - All changes must build successfully, and all tests must complete
  successfully before we can merge each PR. Keep in mind that running tests
  locally can take a loooooong time, so sometimes it can be easier to open your
  PR and let the repo bot check for you.
- **Warnings** - Make sure to address any lingering code warnings before your
  last commit. Keep in mind that sometimes warnings already exist due to changes
  in the compiler's linter between versions. Try to at least make sure that your
  changes do not add any new ones :)
- **Documentation** - If you have made any changes that could benefit from
  updating some code documentation, please be sure to do so! Try to put yourself
  in the shoes of someone reading your code for the first time.
- **Changelog** - [The changelog][nannou-changelog] acts as a human-friendly
  history of the repo that becomes especially useful to community members when
  updating between different versions of nannou. Be sure to add your changes to
  `guide/src/changelog.md`.
- **PR Comment** - Be sure to add the following to your PR to make it easier for
  reviewers to understand and land your code:
    - **Motivation** for changes. What inspired the PR? Please link to any
      related issues.
    - **Summary** of changes. Anything that might help the reviewer to
      understand your what your changes do will go a long way!

If you forget one of these steps before making your PR, don't panic! The nannou
repo has a CI (continuous integration) bot that will check for some of these
steps and notify you if anything is out of order. Once the bot checks pass, a
community member will review the rest.

[nannou-changelog]: https://guide.nannou.cc/changelog.html
