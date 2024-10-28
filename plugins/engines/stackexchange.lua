add_engine('stackoverflow', function (query, _)
	local res = get('https://api.stackexchange.com/2.3/search/advanced?q=' .. query.query .. '&page=' .. query.page .. '&site=stackoverflow', {})

	print(res)
	local data = parse_json(res)

	local results = {}
	for i, item in ipairs(data) do
		results[i] = {
			url = item['link'],
			title = item['title'],
			snippet = table.concat(item['tags'], ' '),
		}
	end

	return results
end)
