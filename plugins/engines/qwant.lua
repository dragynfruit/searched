-- Qwant scraper for Searched
-- Licensed MIT.
-- (c) 2024 Dragynfruit

add_engine('qwant', function(client, query, opts)
	local safesearch = 1 -- default moderate
	if query.safe == 'off' then
		safesearch = 0
	elseif query.safe == 'strict' then
		safesearch = 2
	end

	local url = Url.from_template(
		'https://api.qwant.com/v3/search/web?q={query}&count=10&locale=en_US&offset={offset}&device=desktop&tgp=3&safesearch={safesearch}&displayed=true&llm=false',
		{
			query = query.query,
			offset = tostring((query.page - 1) * 10),
			safesearch = tostring(safesearch),
		}
	):string()

	local data = client
		:req('GET', url)
		:headers({
			['User-Agent'] = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36',
			['Accept'] = 'application/json',
		})
		:send()

	local json = parse_json(data)

	if json.status == 'error' and json.data and json.data.error_code == 27 then
		error('captcha')
	end

	local mainline = json.data.result.items.mainline
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
