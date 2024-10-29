-- -- JSON engine for Searched
-- -- Licensed MIT.
-- -- (c) 2024 Dragynfruit

add_engine('json_engine', function(client, query, opts)
	assert(opts ~= nil, 'fuck')
	assert(type(opts['url']) == 'string', '"url" extra must be set to a string')

	print('aaaaaaa: ' .. opts['url'])

	local url = Url.from_template(tostring(opts['url']), {
		query = query.query,
		page = tostring(query.page),
	}):string()

	print('whu', url)

	local res = client:get(url, {})
	local data = parse_json(res)

	local results = {}
	for i, _ in ipairs(data) do
		if data[i] ~= nil then
			results[i] = {
				title = data[i].Title,
				url = data[i].URL,
				snippet = data[i].Snippet,
			}
		end
	end

	return results
end)
