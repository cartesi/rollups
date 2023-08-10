local function encode_sig(sig)
    local cmd = string.format([[cast sig-event "%s"]], sig)

    local handle = io.popen(cmd)
    assert(handle)

    local encoded_sig = handle:read()
    handle:close()
    return encoded_sig
end

local function decode_event_data(sig, data)
    local cmd = string.format([[cast --abi-decode "bananas()%s" %s]], sig, data)

    local handle = io.popen(cmd)
    assert(handle)

    local decoded_data
    local ret = {}
    repeat
        decoded_data = handle:read()
        table.insert(ret, decoded_data)
    until not decoded_data
    handle:close()
    return ret
end

return {
    encode_sig = encode_sig,
    decode_event_data = decode_event_data,
}
