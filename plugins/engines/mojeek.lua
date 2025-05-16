-- Mojeek scraper for Searched
-- Licensed MIT.
-- (c) 2024 Dragynfruit

add_engine('mojeek', function(client, query, _)
    local offset
    if query.page == 2 then
        offset = (query.page - 1) * 20
    elseif query.page > 2 then
        offset = 20 + (query.page - 2) * 50
    end

    local url = Url.from_template(
        'https://www.mojeek.com/search?q={query}&s={offset}',
        {
            query = query.query,
            offset = tostring(offset),
        }
    ):string()

    local doc = client:req("GET", url)
        :html()

    local links = doc:select('a.title')
    local snippets = doc:select('p.s')

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
            result.general = { snippet = snippet_item.inner_text }
        end

        table.insert(results, result)
    end

    return results
end)
