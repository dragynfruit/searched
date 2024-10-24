-- DuckDuckGo scraper for Searched
-- Licensed MIT.
-- (c) 2024 Dragynfruit

__VQDS = {}

add_search_provider('duckduckgo', function (query)
	local offset
	if query.page == 2 then
		offset = (query.page - 1) * 20
	elseif query.page > 2 then
		offset = 20 + (query.page - 2) * 50
	end

	local headers = { q = query.query }
	if query.page > 1 then
		headers = headers .. {
			s = string(offset),
			nextParams = '',
			v = 'l',
			o = 'json',
			dc = string(offset + 1),
			api = 'd.js',
			vqd = __VQDS[query.query],
			kl = 'wt-wt',
		}
	end

	local res = post('https://lite.duckduckgo.com/lite/', {
		['Content-Type'] = 'application/x-www-form-urlencoded',
		['Referer'] = 'https://lite.duckduckgo.com/',
	}, headers)

	local scr = Scraper.new(res)

	assert(scr ~= nil)

	if __VQDS[query.query] == nil then
		__VQDS[query.query] = scr:select('input[name=vqd]')[1]:attr('value')
	end

	local links = scr:select('a.result-link')
	local snippets = scr:select('td.result-snippet')

	assert(table.pack(links).n == table.pack(snippets).n, 'snippets bronken')

	local ret = {}

	for i, link in ipairs(links) do
		local url = link:attr('href')
		local title = link.inner_html
		local snippet = snippets[i].inner_html

		ret[i] = {
			url = url,
			title = title,
			snippet = snippet,
		}
	end

	return ret
end)
