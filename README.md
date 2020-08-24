<div align="center">

  <h1><code>je</code></h1>

  <h3>
    <strong>Jcr Exchange</strong>
  </h3>

  <p>
    <img src="https://github.com/devzbysiu/je/workflows/Main/badge.svg" alt="CI status
    badge" />
    <a href="https://crates.io/crates/je">
      <img src="https://img.shields.io/crates/v/je?style=for-the-badge" alt="Crates.io version" />
    </a>
    <a href="https://codecov.io/gh/devzbysiu/je">
      <img src="https://img.shields.io/codecov/c/github/devzbysiu/je?style=for-the-badge&token=bfdc5b9d55534910ae48fba0b8e984d0" alt="Code coverage"/>
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

**It's intended to be used as an external tool for IntelliJ IDEA to allow to easily synchronize
content.**

```bash
❯ je
je 0.1.0
Jcr Exchange - easy download and upload files to and from JCR

USAGE:
    je [FLAGS] <SUBCOMMAND>

FLAGS:
    -d, --debug      If enabled, deployed to AEM packages are left intact (are not deleted) to allow investigation
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Enables logs: -v - enables INFO log level -vv - enables DEBUG log level

SUBCOMMANDS:
    get     Download content to local file system
    help    Prints this message or the help of the given subcommand(s)
    init
    put     Upload content to AEM instance

```

# <p id="installation">Installation</p>

### Rust programmers:
```bash
cargo install je
```

### Linux users:
- got to [releases](https://github.com/devzbysiu/je/releases) page
- download the latest Linux executable
- put it into your PATH variable

### Windows users:
- got to [releases](https://github.com/devzbysiu/je/releases) page
- download the latest Windows executable
- put it into your PATH variable

# <p id="configuration">Configuration</p>

### Default
Configuration file is **not** required. Without it, `je` will use default configuration.
However, you can still initialize config and change it. The default configuration is also the initial
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

### IntelliJ Setup

#### Add `je` commands:

1. Go to `Settings -> Tools -> External Tools`.
2. Add new external tool using `+` sign.
3. Configure the tool like on the screenshot below.

![je get configuration](./res/je-get.png)

Similarly add and configure `je put` command:

![je get configuration](./res/je-put.png)


##### Notes
- if you don't have `je` in PATH, you can set full path in `Program` input
- `Arguments` input:
  - `-vv` - sets verbose level, `-vv` means DEBUG log level, `-v` means INFO log level, you can omit
    this option if you don't need logs
  - you can add `-d` to set debug mode in which temporary packages uploaded to AEM won't be deleted
    to allow validation during debugging
  - for more options run `je` in command line
  - subcommand (put or get)
  - `$FilePath$` - IntelliJ variable which will be substituted during command execution, its absolute
    path to a file on which command is executed

#### Configure keyboard mappings

--- TODO ---

# <p id="license">License</p>

This project is licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

# <p id="contribution">Contribution</p>


Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
