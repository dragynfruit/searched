-- Google scraper for Searched
-- Licensed MIT.
-- (c) 2024 Dragynfruit

add_engine('google', function(client, query, _)
	local offset = (query.page - 1) * 10

	local url = Url.parse_with_params(
		--tostring(
		--	'https://google.com/search?filter=0&asearch=arc&oe=utf8&async=use_ac:true,_fmt:prog&start={start}&q={query}'
		--),
		'https://google.com/search',
		{
			filter = '0',
			asearch = 'arc',
			oe = 'utf8',
			async = 'use_ac:true,_fmt:prog',
			start = tostring(offset),
			q = query.query,
		}
	):string()

	local res = client:get(url, {
		['Referer'] = 'https://google.com/',
	})
	local scr = Scraper.new(res)
	assert(scr ~= nil)

	local links = scr:select('a[jsname=UWckNb]')
	local titles = scr:select('a[jsname=UWckNb]>h3')
	local snippets = scr:select('.VwiC3b')

	assert(#links == #titles, 'titles broken')
	assert(#links == #snippets, 'snippets broken')

	--- @type [Result]
	local ret = {}

	for i, _ in ipairs(links) do
		ret[i] = {
			url = links[i]:attr('href'),
			title = titles[i].inner_html,
			general = {
				snippet = snippets[i].inner_html,
			},
		}
	end

	return ret
end)
