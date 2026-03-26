local M = {}

--- Default configuration
M.config = {
  cmd = { "dust-lsp" },
  filetypes = { "dust" },
  root_markers = { "Cargo.toml", "dust.dust", ".git" },
}

--- Setup the Dust LSP client.
--- Works with both nvim-lspconfig and the built-in vim.lsp (Neovim 0.11+).
---
--- Usage with nvim-lspconfig:
---   require("dust-lsp").setup_lspconfig()
---
--- Usage with vim.lsp.config (Neovim 0.11+):
---   require("dust-lsp").setup()
---
--- Manual / older Neovim:
---   require("dust-lsp").setup_manual()
---
---@param opts? table  Override default config fields
function M.setup(opts)
  opts = vim.tbl_deep_extend("force", M.config, opts or {})

  -- Neovim 0.11+ has vim.lsp.config
  if vim.lsp and vim.lsp.config then
    vim.lsp.config("dust_lsp", {
      cmd = opts.cmd,
      filetypes = opts.filetypes,
      root_markers = opts.root_markers,
    })
    vim.lsp.enable("dust_lsp")
    return
  end

  -- Fallback to manual autocommand setup
  M.setup_manual(opts)
end

--- Setup via nvim-lspconfig plugin.
---@param opts? table  Override default config fields
function M.setup_lspconfig(opts)
  opts = vim.tbl_deep_extend("force", M.config, opts or {})

  local lspconfig = require("lspconfig")
  local configs = require("lspconfig.configs")

  if not configs.dust_lsp then
    configs.dust_lsp = {
      default_config = {
        cmd = opts.cmd,
        filetypes = opts.filetypes,
        root_dir = lspconfig.util.root_pattern(unpack(opts.root_markers)),
        settings = {},
      },
    }
  end

  lspconfig.dust_lsp.setup({})
end

--- Manual setup using vim.lsp.start (works on Neovim 0.8+).
---@param opts? table  Override default config fields
function M.setup_manual(opts)
  opts = vim.tbl_deep_extend("force", M.config, opts or {})

  vim.api.nvim_create_autocmd("FileType", {
    pattern = opts.filetypes,
    callback = function(ev)
      vim.lsp.start({
        name = "dust-lsp",
        cmd = opts.cmd,
        root_dir = vim.fs.dirname(
          vim.fs.find(opts.root_markers, {
            upward = true,
            path = vim.fs.dirname(ev.match),
          })[1]
        ),
      })
    end,
  })
end

return M
