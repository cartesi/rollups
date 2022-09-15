local args = {...}
assert(#args <= 1)

local enum_data = require "code-generation-scripts/enum-data/all_data"

local write = args[1]
if not write then
  print("Dry run...\n")
  print("Directories that would be overriden:")
else
    assert(write == "--write")
end

local generate = require "code-generation-scripts/generate_enum_proc"

for k,v in ipairs(enum_data) do
    local out_source_dir = v.out_source_dir .. "/" .. v.data.name .. ".sol"
    local s = generate.source(v.data, v.data_file, out_source_dir)

    if not write then
        print("", out_source_dir)
    else
        local f = assert(io.open(out_source_dir, "w"))
        f:write(s)
        f:close()
    end
end
