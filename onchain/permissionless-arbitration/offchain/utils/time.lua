local clock = os.clock

function sleep(number_of_seconds)
    local t0 = clock()
    while clock() - t0 <= number_of_seconds do end
end

return {sleep = sleep}
