local color = require "utils.color"

local names = {'green', 'yellow', 'blue', 'pink', 'cyan', 'white'}

function log(player_index, msg)
    local color_index = player_index % #names
    local timestamp = os.date("%m/%d/%Y %X")
    print(color.reset .. color.fg[names[color_index]] .. string.format("[#%d][%s] %s", player_index, timestamp, msg) .. color.reset)
end

function log_to_ts(reader, last_ts)
    -- print everything hold in the buffer which has smaller timestamp
    -- this is to synchronise when there're gaps in between the logs
    while true do
        local msg = reader:read()
        if msg then
            print(msg)

            local ts_position = msg:find("%d%d/%d%d/%d%d%d%d %d%d:%d%d:%d%d")
            if ts_position then
                local timestamp = msg:sub(ts_position)
                if timestamp > last_ts then
                    last_ts = timestamp
                    break
                end
            end
        else
            break
        end
    end

    return last_ts
end

return { log = log, log_to_ts = log_to_ts }
