add_engine('stackoverflow', function(client, query, opts)
	local res = client:get(
		'https://api.stackexchange.com/2.3/search/advanced?q='
			.. query.query
			.. '&page='
			.. query.page
			.. '&site='
			.. opts.site,
		{}
	)

	print(res)
	local data = parse_json(res)

	--- @type [Result]
	local results = {}
	for i, item in ipairs(data) do
		results[i] = {
			url = item['link'],
			title = item['title'],
			general = {
				snippet = table.concat(item['tags'], ' '),
			},
		}
	end

	return results
end)
