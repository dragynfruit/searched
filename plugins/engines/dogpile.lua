-- Dogpile scraper for Searched
-- Licensed MIT.
-- (c) 2024 Dragynfruit

add_engine('dogpile', function(client, query, _)
    local doc = client:req("GET", 'https://www.dogpile.com/')
        :html()

    local sc = doc:select('#sc')
    if not sc then
        error("Failed to retrieve search results; sc is nil")
    end

    local url = Url.from_template(
        'https://www.dogpile.com/serp?q={query}&page={page}&sc={sc}',
        {
            query = query.query,
            page = tostring(query.page),
            sc = sc[1]:attr('value'),
        }
    ):string()

    local doc = client:req("GET", url)
        :html()

    local links = doc:select('.web-yahoo__title')
    local snippets = doc:select('.web-yahoo__description')

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
