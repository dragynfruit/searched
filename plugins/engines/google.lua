-- Google scraper for Searched
-- Licensed MIT.
-- (c) 2024 Dragynfruit

add_engine('google', function(client, query, _)
    local offset = query.page * 10

    local url = Url.parse_with_params(
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

    local doc = client:req("GET", url)
        :headers({
            ['Referer'] = 'https://google.com/',
            ['User-Agent'] = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36'
        })
        :html()

    local links = doc:select('a[jsname=UWckNb]')
    local titles = doc:select('a[jsname=UWckNb]>h3')
    local snippets = doc:select('.VwiC3b')

    if not links or not titles or not snippets then
        error("Failed to retrieve search results")
    end

    assert(#links == #titles, 'titles broken')
    assert(#links == #snippets, 'snippets broken')

    local results = {}
    for i, _ in ipairs(links) do
        results[i] = {
            url = links[i]:attr('href'),
            title = titles[i].inner_html,
            general = {
                snippet = snippets[i].inner_text,
            },
        }
    end

    return results
end)
