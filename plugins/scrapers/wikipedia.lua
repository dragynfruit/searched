add_search_provider('wikipedia', function (query)
	local res = get('https://en.wikipedia.org/w/api.php?action=opensearch&format=json&limit=10&namespace=0&search='..query.query, {})

	local data = parse_json(res)

	local results = {}
	if data[1] ~= nil then
		for i, _ in ipairs(data[1]) do
			results[i] = {
				title = data[1],
				url = data[3][i],
			}
		end
	end

	return results
end)
