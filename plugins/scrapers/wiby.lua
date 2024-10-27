-- -- Wiby scraper for Searched
-- -- Licensed MIT.
-- -- (c) 2024 Dragynfruit

add_search_provider('wiby', 'sear', function (query)
    return use_engine('json_engine')('https://wiby.me/json?q=%s&page=%d', query)
end)