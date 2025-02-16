-- -- JSON engine for Searched
-- -- Licensed MIT.
-- -- (c) 2024 Dragynfruit

add_engine('qwant', function(client, query, opts)
	local safesearch = 1  -- default moderate
	if query.safe == "off" then
		safesearch = 0
	elseif query.safe == "strict" then
		safesearch = 2
	end

	local url = Url.from_template('https://api.qwant.com/v3/search/web?q={query}&count=10&locale=en_US&offset={offset}&device=desktop&tgp=3&safesearch={safesearch}&displayed=true&llm=false', {
		query = query.query,
		offset = tostring((query.page - 1) * 10),
		safesearch = tostring(safesearch)
	}):string()

	local res = client:get(url, {})
	local data = parse_json(res)
	
	if data.status == "error" and data.data and data.data.error_code == 27 then
		error("captcha")
	end

	local mainline = data.data.result.items.mainline

	local results = {}
	local i = 1
	for _, item in ipairs(mainline) do
		if item.type == 'web' then
			for _, result in ipairs(item.items) do
				results[i] = {
					title = result.title,
					url = result.url,
					general = {
						snippet = result.desc,
					},
				}
				i = i + 1
			end
		end
	end

	return results
end)
