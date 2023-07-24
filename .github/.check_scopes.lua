-- (c) Cartesi and individual authors (see AUTHORS)
-- SPDX-License-Identifier: Apache-2.0 (see LICENSE)

function extractScopesFromFile(filename)
  local scopes = {}
  local file = io.open(filename)
  if not file then
    error("Failed to open file: " .. filename)
  end
  for line in file:lines() do
    local scope = line:match("%S+")
    if scope then
      scopes[scope] = true
    end
  end
  file:close()
  return scopes
end

function extractScopeFromCommit(commitTitle)
  return commitTitle:match("%a+%((.-)%)")
end
  
local filename = ".github/.scopes.txt"
local scopes = extractScopesFromFile(filename)
  
-- Get commit titles using git log command
local handle = io.popen('git log --pretty=format:"%s" $(git merge-base HEAD origin/main)..HEAD')
local commits = handle:read("a")
assert(handle:close())

-- Check if the scope matches for each commit
for line in commits:gmatch("[^\r\n]+") do
  local commitScope = extractScopeFromCommit(line)
  if commitScope and not scopes[commitScope] then
    print("Invalid scope:", commitScope)
    print("Commit:", line)
    os.exit(1)
  end
end
