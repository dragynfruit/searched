--- @meta preserver

--- Add a search provider
---
--- @param name string
--- @param callback fun(query: Query): [Result]
function add_search_provider(name, callback) end

--- Make a GET request
---
--- @param url string
--- @param headers table<string, string>
---
--- @return string
function get(url, headers) end

--- Make a POST request
---
--- @param url string
--- @param headers table<string, string>
--- @param form table<string, string>
---
--- @return string
function post(url, headers, form) end

--- Parse JSON
---
--- @param raw string
--- @return table<string|number, any>
function parse_json(raw) end

--- @class Query
--- A search query
---
--- @field public query string
--- @field public page number
Query = {}

--- @class Result
--- A search result
---
--- @field public url string
--- @field public title string
--- @field public general? string
Result = {}

--- @class Element
--- 
--- @field public inner_html string
Element = {}

--- Get the value of an attribute
---
--- @param attr string
--- @return string
function Element:attr(attr) end

--- @class Scraper
--- An HTML scraper
Scraper = {}

--- Build a new Scraper from the raw HTML document
---
--- @param raw string A raw HTML document
--- @return Scraper
function Scraper.new(raw) end

--- Get elements matching a selector
---
--- @param selector string
--- @return [Element]
function Scraper:select(selector) end


