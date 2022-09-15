
local splice_machine
do
    local data_file = "code-generation-scripts/enum-data/splice_machine"
    local data = require(data_file)
    splice_machine = {
        data = data,
        data_file = data_file,
        out_source_dir = "src/splice",
        out_test_dir = "test/two-party-arbitration",
    }
end

local two_party_arbitration
do
    local data_file = "code-generation-scripts/enum-data/two_party_arbitration"
    local data = require(data_file)
    two_party_arbitration = {
        data = data,
        data_file = data_file,
        out_source_dir = "src/two-party-arbitration",
        out_test_dir = "test/two-party-arbitration",
    }
end

return {
    splice_machine,
    two_party_arbitration,
}
