<div align="center">

  <h1><code>je</code></h1>

  <h3>
    <strong>Jcr Exchange</strong>
  </h3>

  <p>
    <img src="https://github.com/devzbysiu/je/workflows/Main/badge.svg" alt="CI status
    badge" />
    <a href="https://codecov.io/gh/devzbysiu/je">
      <img src="https://img.shields.io/codecov/c/github/devzbysiu/je?style=for-the-badge&token=bfdc4b9d55534910ae48fba0b8e984d0" alt="Code coverage"/>
    </a>
    <a href="https://crates.io/crates/je">
      <img src="https://img.shields.io/crates/l/je?style=for-the-badge" alt="License"/>
    </a>
    <a href="https://docs.rs/je">
      <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=for-the-badge" alt="docs.rs docs" />
    </a>
  </p>

  <h3>
    <a href="#about">About</a>
    <span> | </span>
    <a href="#installation">Installation</a>
    <span> | </span>
    <a href="#configuration">Configuration</a>
    <span> | </span>
    <a href="#license">License</a>
    <span> | </span>
    <a href="#contribution">Contribution</a>
  </h3>

  <sub><h4>Built with 🦀</h4></sub>
</div>

# <p id="about">About</p>

Small utility for uploading/downloading content to/from running AEM instance.

It's intended to be used as an external tool for IntelliJ Idea to allow to easily synchronize
content.

# <p id="installation">Installation</p>

Rust programmers:
```bash
cargo install je
```

# <p id="configuration">Configuration</p>

### Default
Configuration file is **not** required. Without it, `je` will use default configuration.
However, you can still initialize config and change it. The default configuration is the initial
one:

```bash
$ je init
$ cat .je
ignore_properties = []

[instance]
addr = "http://localhost:4502"
user = "admin"
pass = "admin"
```

### Customize
`ignore_properties` - tell `je` which properties of `.content.xml` should be removed after
downloading the content

`addr` - address of the instance, including port if domain is not available

`user` - user used to authenticate to AEM instance

`pass` - password used to authenticate to AEM instance


# <p id="license">License</p>

This project is licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

# <p id="contribution">Contribution</p>


Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
