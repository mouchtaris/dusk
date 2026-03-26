# Dust language support for Neovim

Syntax highlighting, filetype detection, and LSP integration for `.dust` files.

## Install

### lazy.nvim

```lua
{
  dir = "/path/to/dusk/editors/nvim",
  ft = "dust",
  config = function()
    require("dust-lsp").setup()
  end,
}
```

### vim-plug

```vim
Plug '/path/to/dusk/editors/nvim'
```

Then in your Lua config:

```lua
require("dust-lsp").setup()
```

### Manual (symlink)

```sh
ln -s /path/to/dusk/editors/nvim ~/.config/nvim/pack/dust/start/dust-nvim
```

## LSP Setup

First, build and install the language server:

```sh
cd /path/to/dusk/m/dust-lsp
cargo install --path .
```

Then choose one of these approaches in your Neovim config:

### Neovim 0.11+ (built-in vim.lsp.config)

```lua
require("dust-lsp").setup()
```

### nvim-lspconfig

```lua
require("dust-lsp").setup_lspconfig()
```

### Neovim 0.8+ (manual)

```lua
require("dust-lsp").setup_manual()
```

### Custom binary path

```lua
require("dust-lsp").setup({
  cmd = { "/path/to/dust-lsp" },
})
```

## Features

- **Syntax highlighting**: keywords, system commands (`!cmd`), variables (`$var`),
  strings, raw strings (`r#"..."#`), paths, options, comments, numbers
- **Filetype detection**: automatic for `.dust` files
- **Filetype settings**: comment format, indentation
- **LSP**: diagnostics, hover, go-to-definition
