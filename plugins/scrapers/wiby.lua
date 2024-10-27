-- -- Wiby scraper for Searched
-- -- Licensed MIT.
-- -- (c) 2024 Dragynfruit

add_search_provider('wiby', 'sear', function (query)
    print('Searching Wiby for ' .. query.query)
    use_engine('json_engine')('https://wiby.me/api/search?q=%s&page=%d', query)
end)