-- Searched Lua API Definition
-- Licensed MIT.
-- (c) 2024 Dragynfruit

--- @meta searched

--- Add a ranker
---
--- @param name string
--- @param callback fun(results: Result[], options: table<string, string|number|boolean>): number[]
function add_ranker(name, callback) end

--- Add a merger
---
--- @param name string
--- @param callback fun(results: Result[], options: table<string, string|number|boolean>): Result[]
function add_merger(name, callback) end
