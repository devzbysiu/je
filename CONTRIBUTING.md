# Contribution guidelines

First off, thank you for considering contributing to `je`.

If your contribution is not straightforward, please first discuss the change you
wish to make by creating a new issue before making the change.

One of the project goals is to be easy to understand so, especially for github
actions, try to keep things simple and to add comments whenever this is not
possible.

## Reporting issues

Issues have to be reported on our [issues tracker](https://github.com/devzbysiu/je/issues). Please:

- Check that the issue has not already been reported.
  - This can be achieved by searching keywords on the [issues tracker](https://github.com/devzbysiu/je/issues).
- Try to use a clear title, and describe your problem with complete sentences.

## Pull requests

Try to do one pull request per change.

### Updating the changelog

Update the changes you have made in
[CHANGELOG](https://github.com/devzbysiu/je/blob/master/CHANGELOG.org)
file under the **Unreleased** section.

Add the changes of your pull request to one of the following subsections,
depending on the types of changes defined by
[Keep a changelog](https://keepachangelog.com/en/1.0.0/):

- `Added` for new features.
- `Changed` for changes in existing functionality.
- `Deprecated` for soon-to-be removed features.
- `Removed` for now removed features.
- `Fixed` for any bug fixes.
- `Security` in case of vulnerabilities.

If the required subsection does not exist yet under **Unreleased**, create it!

## Developing

### Set up

This is no different than other Rust projects.

```shell
git clone https://github.com/devzbysiu/je
cd je
cargo build
```

### Useful Commands

- Build and run release version:

  ```shell
  cargo build --release && cargo run --release
  ```

- Run Clippy:

  ```shell
  cargo clippy --all-targets --all-features -- -W clippy::pedantic -D warnings
  ```

- Run all tests:

  ```shell
  cargo test --all
  ```

- Check to see if there are code formatting issues

  ```shell
  cargo fmt --all -- --check
  ```

- Format the code in the project

  ```shell
  cargo fmt --all
  ```
