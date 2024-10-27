-- -- JSON engine for Searched
-- -- Licensed MIT.
-- -- (c) 2024 Dragynfruit

add_engine("json_engine", function(url, query) 
    local res = get(string.format(url, query.query, query.page), {})
    local data = parse_json(res)

    local results = {}
	-- if data[2] ~= nil then
	-- 	for i, _ in ipairs(data[2]) do
	-- 		if data[3] ~= nil then
	-- 			results[i] = {
	-- 				title = data[2][i],
	-- 				url = data[3][i],
	-- 			}
	-- 		end
	-- 	end
	-- end
    for i, _ in ipairs(data) do
        results[i] = {
            title = data[i].title,
            url = data[i].url,
            snippet = data[i].snippet
        }
    end

	return results
end)