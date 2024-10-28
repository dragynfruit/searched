-- Searched Lua API Definition
-- Licensed MIT.
-- (c) 2024 Dragynfruit

--- @meta searched

--- Add a search provider
---
--- @param name string
--- @param kind Kind
--- @param callback fun(query: Query): [Result]
function add_search_provider(name, kind, callback) end

--- Add a engine
---
--- @param name string
--- @param callback fun(query: Query, url: string|nil): [Result]
function add_engine(name, callback) end

--- Use a engine
---
--- @param name string
---
--- @return fun(url: string, query: Query): [Result]
function use_engine(name) end

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

--- Stringify parameters
--- 
--- @param params table<string, string>
--- 
--- @return string
function stringify_params(params) end

--- Parse JSON
---
--- @param raw string
--- @return table<string|number, any>
function parse_json(raw) end

--- @enum Kind
Kind = {
	General        = 'sear',
	Images         = 'imgs',
	Videos         = 'vids',
	News           = 'news',
	Maps           = 'maps',
	Wiki           = 'wiki',
	QuestionAnswer = 'qans',
	Documentation  = 'docs',
	Papers         = 'pprs',
}

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
--- @field public snippet? string
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


