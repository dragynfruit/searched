-- Startpage scraper for Searched
-- Licensed MIT.
-- (c) 2024 Dragynfruit

add_engine('startpage', function(client, query, _)
	local safesearch = 'none'
	if query.safe == 'off' then
		safesearch = 'none'
	elseif query.safe == 'strict' then
		safesearch = 'heavy'
	end

	local url = Url.from_template('https://www.startpage.com/do/search?query={query}&page={page}&qadf={safe}', {
		query = query.query,
		page = tostring(query.page),
		safe = safesearch,
	}):string()

	local doc = client:req('GET', url):html()

	local links = doc:select('.result a.result-title')
	local snippets = doc:select('.result p.description')

	assert(#links == #snippets, 'snippets broken')

	local results = {}
	for i, link in ipairs(links) do
		local url = link:attr('href')
		local title = link.inner_html
		local result = {
			url = url,
			title = title,
		}

		local snippet_item = snippets[i]
		if snippet_item then
			result.general = { snippet = snippet_item.inner_html }
		end

		table.insert(results, result)
	end

	return results
end)
