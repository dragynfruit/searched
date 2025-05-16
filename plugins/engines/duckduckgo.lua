-- DuckDuckGo scraper for Searched
-- Licensed MIT.
-- (c) 2024 Dragynfruit

--- Checks if a URL is an advertisement
--- @param url string
--- @return boolean
local function is_ad_url(url)
    return string.match(url, "duckduckgo.com.*ad*") ~= nil
end

add_engine('duckduckgo', function(client, query, _)
    local offset
    if query.page == 2 then
        offset = (query.page - 1) * 20
    elseif query.page > 2 then
        offset = 20 + (query.page - 2) * 50
    end

    local form_data = { q = query.query }
    if query.page > 1 then
        form_data = {
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

    local doc = client:req("POST", 'https://html.duckduckgo.com/html')
        :headers({
            ['Content-Type'] = 'application/x-www-form-urlencoded',
            ['Referer'] = 'https://lite.duckduckgo.com/'
        })
        :form(form_data)
        :html()

    local links = doc:select('a.result__a')
    local snippets = doc:select('a.result__snippet')

    assert(#links == #snippets, 'snippets broken')

    local results = {}
    for i, link in ipairs(links) do
        local url = link:attr('href')
        if not is_ad_url(url) then
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
    end

    return results
end)
