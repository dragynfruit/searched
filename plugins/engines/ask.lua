-- Ask.com scraper for Searched
-- Licensed MIT.
-- (c) 2024 Dragynfruit

add_engine('ask', function(client, query, _)
	local url = Url.from_template('https://www.ask.com/web?o=0&qo=pagination&page={page}&q={query}', {
		query = query.query,
		page = tostring(query.page),
	}):string()

	local html = client:req('GET', url):send()

	-- Extract JSON using script tag pattern
	local json_str = html:match('window%.MESON%.initialState%s*=%s*({.-});%s*window%.MESON%.loadedLang')
	if not json_str then
		error('Could not find MESON.initialState')
	end

	local data = parse_json(json_str)

	-- Extract results from the parsed JSON
	local web_results = data.search.webResults.results
	if not web_results then
		error('No results found')
	end

	local results = {}
	for i, result in ipairs(web_results) do
		if result.url then
			results[i] = {
				url = result.url,
				title = result.title,
				general = {
					snippet = result.abstract,
				},
			}
		end
	end

	return results
end)
