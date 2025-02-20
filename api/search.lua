-- Searched Lua API Definition
-- Licensed MIT.
-- (c) 2024 Dragynfruit

--- @meta searched

--- Add a engine
---
--- @param name string
--- @param callback fun(client: Client, query: Query, options: table<string, string|number|boolean>): [Result]
function add_engine(name, callback) end

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
	General = 'sear',
	Images = 'imgs',
	Videos = 'vids',
	News = 'news',
	Maps = 'maps',
	Wiki = 'wiki',
	QuestionAnswer = 'qans',
	Documentation = 'docs',
	Papers = 'pprs',
}

--- @class Query
--- A search query
---
--- @field public query string
--- @field public page number
--- @field public safe string
Query = {}

--- @class Element
---
--- @field public inner_html string
Element = {}

--- Get the value of an attribute
---
--- @param attr string
--- @return string
function Element:attr(attr) end

--- @class Client
--- An HTTP client
Client = {}

--- Start building a request
---
--- @param method string HTTP method (GET, POST etc)
--- @param url string
--- @return RequestBuilder
function Client:req(method, url) end

--- @class RequestBuilder
--- A request builder
RequestBuilder = {}

--- Set request headers
---
--- @param headers table<string, string>
--- @return RequestBuilder
function RequestBuilder:headers(headers) end

--- Set form data
---
--- @param form table<string, string>
--- @return RequestBuilder
function RequestBuilder:form(form) end

--- Set JSON body
---
--- @param body table
--- @return RequestBuilder
function RequestBuilder:json(body) end

--- Execute the request
---
--- @return Response
function RequestBuilder:send() end

--- @class Response
--- A response from a request
Response = {}

--- Convert response to HTML document
--- @return HtmlDocument
function Response:html() end

--- Parse response as JSON
--- @return table
function Response:json() end

--- Convert response to HTML document
--- @return HtmlDocument
function RequestBuilder:html() end

--- @class HtmlDocument
--- Raw HTML document
HtmlDocument = {}

--- Create new document from string
--- @param html string Raw HTML content
--- @return HtmlDocument
function HtmlDocument.from_string(html) end

--- Select elements from document
--- @param selector string CSS selector
--- @return [Element]
function HtmlDocument:select(selector) end

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
