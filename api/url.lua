-- Searched Lua API Definition
-- Licensed MIT.
-- (c) 2024 Dragynfruit

--- @meta searched

--- @class Url
Url = {}

--- Build a Url from a template string
---
--- @param template string
--- @param values table<string, string>
---
--- @return Url
function Url.from_template(template, values) end

--- Get the Url as a string
---
--- @return string
function Url:string() end
