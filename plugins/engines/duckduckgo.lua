-- DuckDuckGo scraper for Searched
-- Licensed MIT.
-- (c) 2024 Dragynfruit

add_engine('duckduckgo', function(client, query, _)
	local offset
	if query.page == 2 then
		offset = (query.page - 1) * 20
	elseif query.page > 2 then
		offset = 20 + (query.page - 2) * 50
	end

	local headers = { q = query.query }
	if query.page > 1 then
		headers = {
			q = query.query,
			s = tostring(offset),
			nextParams = '',
			v = 'l',
			o = 'json',
			dc = tostring(offset + 1),
			api = 'd.js',
			vqd = '',
			kl = 'wt-wt',
		}
	end

	local res = client:post('https://lite.duckduckgo.com/lite/', {
		['Content-Type'] = 'application/x-www-form-urlencoded',
		['Referer'] = 'https://lite.duckduckgo.com/',
	}, headers)

	if not res then
		error("Failed to get response from DuckDuckGo")
	end

	local scr = Scraper.new(res)
	if not scr then
		error("Failed to create scraper from response")
	end

	-- TODO: need to add vqd handling

	local links = scr:select('a.result-link')
	local snippets = scr:select('td.result-snippet')

    if not links or not snippets then
        error("Failed to retrieve search results; links or snippets are nil")
    end

	assert(table.pack(links).n == table.pack(snippets).n, 'snippets bronken')

	--- @type [Result]
	local ret = {}

	for i, link in ipairs(links) do
		local url = link:attr('href')
		local title = link.inner_html
		local ret_item = {
			url = url,
			title = title,
		}
		local snippet_item = snippets[i]
		if snippet_item then
			ret_item.general = { snippet = snippet_item.inner_html }
		end

		ret[i] = ret_item
	end

	return ret
end)
