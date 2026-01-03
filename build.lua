local function notify(msg, level)
  coroutine.yield({
    msg = msg,
    level = level or vim.log.levels.TRACE,
  })
end

local function lib_names()
  local os_name = vim.uv.os_uname().sysname

  if os_name == "Linux" then
    return "libbight.so", "bight.so"
  elseif os_name == "Darwin" then
    return "libbight.dylib", "bight.so"
  elseif os_name:match("Windows") then
    return "bight.dll", "bight.dll"
  else
    notify("Unsupported OS: " ..
      os_name .. ". If you want to add support, open a PR at https://github.com/WASDetchan/bight.nvim",
      vim.log.levels.ERROR)
  end
end

local function build()
  local root = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p:h")

  local src_name, dst_name = lib_names()

  if not src_name then return end

  notify("Building (cargo build --release)…")

  local err = nil

  vim.system(
    { "cargo", "build", "--release" },
    { cwd = root, text = true },
    function(res)
      if res.code ~= 0 then
        err = res.stderr or "cargo build failed"
        return
      end
    end
  ):wait()
  if err then
    notify(err, vim.log.levels.ERROR)
  else
    notify("Built " .. dst_name)
  end

  local src = root .. "/target/release/" .. src_name
  local dst_dir = root .. "/lua"
  local dst = dst_dir .. "/" .. dst_name

  if vim.fn.filereadable(src) == 0 then
    notify("Build succeeded but output not found: " .. src, vim.log.levels.ERROR)
    return
  end

  print("dst_dir: ", dst_dir)
  vim.fn.mkdir(dst_dir, "p")
  vim.fn.delete(dst)
  vim.fn.writefile(vim.fn.readfile(src, "b"), dst, "b")

  notify("Installed " .. dst_name)
end

build()
