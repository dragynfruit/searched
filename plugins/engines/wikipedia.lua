-- -- Wikipedia scraper for Searched
-- -- Licensed MIT.
-- -- (c) 2024 Dragynfruit

add_engine("mediawiki", function(client, query, _)
	local res = client:get(
		"https://en.wikipedia.org/w/api.php?action=opensearch&format=json&limit=10&namespace=0&search=" .. query.query,
		{}
	)

	local data = parse_json(res)

	local results = {}
	if data[2] ~= nil then
		for i, _ in ipairs(data[2]) do
			if data[3] ~= nil then
				results[i] = {
					title = data[2][i],
					url = data[3][i],
				}
			end
		end
	end

	return results
end)
