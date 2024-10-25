-- -- Google scraper for Searched
-- -- Licensed MIT.
-- -- (c) 2024 Dragynfruit

-- __VQDS = {}

-- add_search_provider('google', function (query)
-- 	local offset = (query.page - 1) * 10
-- 	local params = { q = query.query, filter = 0, start = offset, asearch = 'arc', async = 'use_ac:true,_fmt:prog' }

-- 	local res = get('https://google.com/search/?' .. stringify_params(params), {
-- 		['Referer'] = 'https://google.com/',
-- 	})

-- 	local scr = Scraper.new(res)
-- 	assert(scr ~= nil)

-- 	if __VQDS[query.query] == nil then
-- 		__VQDS[query.query] = scr:select('input[name=vqd]')[1]:attr('value')
-- 	end

-- 	-- local links = scr:select('a.result-link')
-- 	-- local snippets = scr:select('td.result-snippet')

-- 	-- assert(table.pack(links).n == table.pack(snippets).n, 'snippets bronken')

-- 	-- local ret = {}

-- 	-- for i, link in ipairs(links) do
-- 	-- 	local url = link:attr('href')
-- 	-- 	local title = link.inner_html
-- 	-- 	local snippet = snippets[i].inner_html

-- 	-- 	ret[i] = {
-- 	-- 		url = url,
-- 	-- 		title = title,
-- 	-- 		snippet = snippet,
-- 	-- 	}
-- 	-- end

-- 	-- return ret
-- end)
