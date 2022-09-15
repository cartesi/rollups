local args = {...}
assert(#args == 2)

local enum_data_file = args[1]
local out_dir = args[2]
local enum_data = require(enum_data_file)

local generate_enum = require "code-generation-scripts/generate_enum_proc"

local s = generate_enum(enum_data, enum_data_file, out_dir)
local f = assert(io.open(out_dir, "w"))
f:write(s)
f:close()
