-- Yahoo scraper for Searched
-- Licensed MIT.
-- (c) 2024 Dragynfruit

--- Unquote the url
--- @param url string
--- @return string
local function unquote(url)
	return url:gsub('%%(%x%x)', function(hex)
		return string.char(tonumber(hex, 16))
	end)
end

--- Converts the yahoo tracking url to a normal url
--- @param url_string string
--- @return string
local function remove_tracking(url_string)
	local start = url_string:find('RU=') + 3
	url_string = url_string:sub(start)
	local end_ = url_string:find('/')
	url_string = url_string:sub(1, end_ - 1)
	return unquote(url_string)
end

add_engine('yahoo', function(client, query, opts)
	local offset = query.page * 7 + 1

	local url = Url.from_template('https://search.yahoo.com/search?ei=UTF-8&o=json&p={query}&b={offset}', {
		query = query.query,
		offset = tostring(offset),
	}):string()

	local data = client
		:req('GET', url)
		:headers({
			['Referer'] = 'https://yahoo.com/',
		})
		:send()

	local doc = HtmlDocument.from_string(parse_json(data).body)

	local links = doc:select('.title>a')
	local snippets = doc:select('.compText>p')

	assert(#links == #snippets, 'snippets broken')

	local results = {}
	for i, _ in ipairs(links) do
		local result_url = remove_tracking(links[i]:attr('href'))

		if result_url ~= nil then
			results[i] = {
				url = result_url,
				title = links[i]:attr('aria-label'),
				general = {
					snippet = snippets[i].inner_html,
				},
			}
		end
	end

	return results
end)
