-- Stract scraper for Searched
-- Licensed MIT.
-- (c) 2024 Dragynfruit

add_engine('stract', function(client, query, opts)
    -- Build request body
    local body = {
        query = query.query,
        page = query.page - 1, -- Stract uses 0-based pagination
        numResults = 10,
        flattenResponse = true,
        countResultsExact = true,
        safeSearch = query.safe ~= "off"
    }

    local res = client:req("POST", 'https://stract.com/beta/api/search')
        :headers({
            ['Content-Type'] = 'application/json'
        })
        :json(body)
        :send()

    local data = parse_json(res)

    if not data or not data.webpages then
        error("Failed to get valid response from Stract")
    end

    local results = {}
    for i, result in ipairs(data.webpages) do
        -- Combine snippet fragments
        local snippet = ""
        if result.snippet and result.snippet.text and result.snippet.text.fragments then
            for _, fragment in ipairs(result.snippet.text.fragments) do
                snippet = snippet .. fragment.text
            end
        end

        results[i] = {
            url = result.url,
            title = result.title,
            general = {
                snippet = snippet,
            },
        }
    end

    return results
end)