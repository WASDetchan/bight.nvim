local function notify(msg, level)
  -- print("notified: ", msg)
  coroutine.yield({
    msg = msg,
    level = level or vim.log.levels.INFO,
  })
end

local to_notify = {}

local function notify_add(msg, level)
  table.insert(to_notify, { msg = msg, level = level or vim.log.levels.INFO })
end

local function notify_all()
  local tab
  to_notify, tab = {}, to_notify
  for _, msg in pairs(tab) do
    -- print("notifiying all: msg = ", msg.msg)
    notify(msg.msg, msg.level)
  end
  -- print("notified all")
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

  notify("Building (cargo build --release --locked)…")

  local uv = vim.uv
  local stderr = uv.new_pipe()
  local return_code = nil

  local handle, _ = uv.spawn(
    "cargo",
    {
      cwd = root,
      args = { "build", "--release", "--locked" },
      stdio = { nil, nil, stderr }
    },
    function(code, _)
      -- notify_add("Finished building with code " .. code)
      return_code = code
    end
  )

  if not stderr or not handle then
    notify("Failed to spawn build process!", vim.log.levels.ERROR)
    return
  end

  stderr:read_start(function(_, chunk)
    -- print("got chunk of stderr: ", chunk)
    if chunk then
      notify_add(chunk)
    end
  end)

  while not return_code do
    if (#to_notify > 0) then
      notify_all()
      -- print("notified all")
    end
    -- vim.wait(100)
    coroutine.yield()
  end
  notify_all()


  if return_code ~= 0 then
    notify("Build failed with exit code " .. return_code, vim.log.levels.ERROR)
    return
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

  -- print("dst_dir: ", dst_dir)
  vim.fn.mkdir(dst_dir, "p")
  vim.fn.delete(dst)
  vim.fn.writefile(vim.fn.readfile(src, "b"), dst, "b")

  notify("Installed " .. dst_name)
end

build()
