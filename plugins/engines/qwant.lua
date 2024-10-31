-- -- JSON engine for Searched
-- -- Licensed MIT.
-- -- (c) 2024 Dragynfruit

add_engine('qwant', function(client, query, opts)
	local url = Url.from_template('https://api.qwant.com/v3/search/web?locale=en_us&count=10&p={page}&q={query}', {
		query = query.query,
		page = tostring(query.page),
	}):string()

	local res = client:get(url, {})
	local data = parse_json(res)
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
