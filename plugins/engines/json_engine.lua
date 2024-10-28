-- -- JSON engine for Searched
-- -- Licensed MIT.
-- -- (c) 2024 Dragynfruit

add_engine("json_engine", function(client, _, url)
	if url ~= nil then
    local res = client:get(url, {})
    local data = parse_json(res)

    local results = {}
    for i, _ in ipairs(data) do
        results[i] = {
            title = data[i].Title,
            url = data[i].URL,
            snippet = data[i].Snippet
        }
    end

		return results
	else
		print('wtf')
		return {}
	end
end)
