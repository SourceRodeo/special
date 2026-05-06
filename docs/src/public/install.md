@filedocuments spec SPECIAL.DISTRIBUTION.HOMEBREW.INSTALLS_SPECIAL
# Install and Update

Install `special` with Homebrew when possible:

```sh
brew install sourcerodeo/homebrew-tap/special
```

[Cargo is the secondary route](documents://spec/SPECIAL.DISTRIBUTION.CRATES_IO.BINARY_NAME):

```sh
cargo install special-cli
```

Both routes install the `special` executable. Confirm the binary is available:

```sh
special --version
special --help
```

## Repository Setup

Initialize a repository once:

```sh
special init
```

That creates `special.toml` when no active config already exists. In an existing
project, review the generated config before committing it.

## Updating

For Homebrew installs:

```sh
brew update
brew upgrade special
```

For Cargo installs:

```sh
cargo install special-cli --force
```

Run `special lint` after an update if the repository already contains Special
annotations.

## Codex Users

Install the SourceRodeo Codex marketplace, then install the Special plugin from
that marketplace:

```sh
codex plugin marketplace add SourceRodeo/codex-marketplace
```

The plugin gives Codex workflow skills and MCP configuration. It does not replace
the native binary install; the plugin setup skills should still guide users to a
normal Homebrew or Cargo install when `special` is missing.
