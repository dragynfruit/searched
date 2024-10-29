-- -- Wikipedia scraper for Searched
-- -- Licensed MIT.
-- -- (c) 2024 Dragynfruit

add_engine("mediawiki", function(client, _, url)
	if url ~= nil then
		local res = client:get(
			url,
			{}
		)

		local data = parse_json(res)

		local results = {}
		if data[2] ~= nil then
			for i, _ in ipairs(data[2]) do
				if data[4] ~= nil and data[4][i] ~= nil then
					results[i] = {
						title = data[2][i],
						url = data[4][i],
					}
				end
			end
		end

		return results
	else
		return {}
	end
end)
