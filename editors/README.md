# Dust Editor Support

Syntax highlighting and language server (LSP) support for the
[Dust](./doc/tutorial.md) scripting language. Includes plugins for
**VS Code** and **Neovim**.

## Language Server

Both editor plugins use the same LSP server, `dust-lsp`. Build it first.

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)

### Build & install

```sh
cd m/dust-lsp
cargo install --path .
```

This puts `dust-lsp` on your `$PATH` (typically `~/.cargo/bin/dust-lsp`).

Verify it works:

```sh
dust-lsp --version   # should not error
```

### LSP features

| Feature            | Description                                                      |
| ------------------ | ---------------------------------------------------------------- |
| **Diagnostics**    | Mismatched / unclosed brackets, malformed bindings               |
| **Hover**          | Documentation for keywords (`def`, `let`, `val`, `src`, ...),    |
|                    | binding info with execution semantics, system command names      |
| **Go to definition** | Jump from `$variable` or invocation name to its `def`/`let`/`val`/`src` binding |

---

## VS Code

### Install

The extension is not yet published to the marketplace. Install it locally:

```sh
cd editors/vscode
npm install
npm run compile
```

Then either:

- **Symlink** into your VS Code extensions directory:

  ```sh
  ln -s "$(pwd)" ~/.vscode/extensions/dust-lang
  ```

- **Or** open VS Code, run **Developer: Install Extension from Location...**
  and select the `editors/vscode` directory.

Restart VS Code. Any `.dust` file will now have syntax highlighting and
LSP features.

### Configuration

In VS Code settings (`settings.json`):

```jsonc
{
  // Path to the dust-lsp binary (default: "dust-lsp", found via $PATH)
  "dust.lsp.path": "dust-lsp",

  // Set to false to disable the language server (keeps syntax highlighting)
  "dust.lsp.enabled": true
}
```

### What you get

- **Syntax highlighting** via TextMate grammar — keywords, `!system` commands,
  `$variables`, `$slices[0;]`, `*dereferences`, strings, raw strings
  (`r#"..."#`), paths, `--options`, comments, numbers, environment variables
- **Language configuration** — bracket matching & auto-close, `#` comment
  toggling, code folding on `{ }`, smart indentation
- **LSP** — real-time diagnostics, hover, go-to-definition

---

## Neovim

### Install

Choose one of:

#### lazy.nvim

```lua
{
  dir = "/path/to/dusk/editors/nvim",
  ft = "dust",
  config = function()
    require("dust-lsp").setup()
  end,
}
```

#### packer.nvim

```lua
use {
  "/path/to/dusk/editors/nvim",
  ft = "dust",
  config = function()
    require("dust-lsp").setup()
  end,
}
```

#### vim-plug

```vim
Plug '/path/to/dusk/editors/nvim'
```

Then add to your Lua config:

```lua
require("dust-lsp").setup()
```

#### Manual (symlink)

```sh
mkdir -p ~/.config/nvim/pack/dust/start
ln -s /path/to/dusk/editors/nvim ~/.config/nvim/pack/dust/start/dust-nvim
```

Then add to `init.lua`:

```lua
require("dust-lsp").setup()
```

### LSP setup methods

The plugin provides three ways to start the language server, depending on
your Neovim version and plugin setup:

#### Neovim 0.11+ (recommended)

Uses the built-in `vim.lsp.config` / `vim.lsp.enable`:

```lua
require("dust-lsp").setup()
```

#### nvim-lspconfig

Registers `dust_lsp` as a custom server with
[nvim-lspconfig](https://github.com/neovim/nvim-lspconfig):

```lua
require("dust-lsp").setup_lspconfig()
```

#### Neovim 0.8+ (manual)

Creates a `FileType` autocommand that calls `vim.lsp.start`:

```lua
require("dust-lsp").setup_manual()
```

### Custom binary path

All three methods accept an options table:

```lua
require("dust-lsp").setup({
  cmd = { "/path/to/dust-lsp" },
  -- filetypes = { "dust" },
  -- root_markers = { "Cargo.toml", "dust.dust", ".git" },
})
```

### What you get

- **Syntax highlighting** — keywords, `!system` commands, `$variables`,
  `*dereferences`, strings, raw strings (`r#"..."#`), paths, `--options`,
  comments, numbers, `ENV_VAR =` assignments
- **Filetype detection** — automatic for `.dust` files
- **Filetype settings** — `#` comment format, 2-space indentation
- **LSP** — real-time diagnostics, hover, go-to-definition

---

## Syntax at a glance

Here is how the major Dust constructs are highlighted:

```dust
# Comments start with #

include ../lib/dusk.dust;              # keyword + path

def build =                            # keyword + function name
  !cargo build                         # system command
    --bin xs-compile                    # long option
    -v                                 # short option
    $args                              # variable
;

let lib =
  ./target/debug/xs-compile            # relative path
    RUST_LOG = "compile=trace"         # env var + string
    <$input                            # redirect + variable
    $args[1;]                          # slice
;

src output = *compile <$lib;           # dereference + redirect

val x = 42;                            # number
val raw = r#"raw string content"#;     # raw string
```

---

## Project layout

```
editors/
  vscode/
    package.json                 # VS Code extension manifest
    language-configuration.json  # brackets, comments, folding
    syntaxes/
      dust.tmLanguage.json       # TextMate grammar
    src/
      extension.ts               # LSP client
    tsconfig.json
  nvim/
    ftdetect/dust.vim            # filetype detection
    ftplugin/dust.vim            # filetype settings
    syntax/dust.vim              # Vim syntax highlighting
    lua/dust-lsp/init.lua        # LSP client
m/
  dust-lsp/
    Cargo.toml                   # standalone Rust crate
    src/
      main.rs                    # LSP server (lsp-server + lsp-types)
      analysis.rs                # diagnostics, hover, go-to-definition
      analysis/lexer.rs          # Dust tokenizer
```
