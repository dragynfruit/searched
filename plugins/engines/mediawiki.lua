-- -- Wikipedia scraper for Searched
-- -- Licensed MIT.
-- -- (c) 2024 Dragynfruit

add_engine('mediawiki', function(client, query, opts)
	local url = Url.from_template(tostring(opts.url), {
		query = query.query,
	}):string()

	local res = client:get(url, {})
	local data = parse_json(res)

	--- @type [Result]
	local results = {}
	if data[2] ~= nil then
		for i, _ in ipairs(data[2]) do
			if data[4] ~= nil and data[4][i] ~= nil then
				results[i] = {
					title = data[2][i],
					url = data[4][i],
					general = {
						snippet = data[3][i],
					},
				}
			end
		end
	end

	return results
end)
