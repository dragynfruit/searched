-- Searched Lua API Definition
-- Licensed MIT.
-- (c) 2024 Dragynfruit

--- @meta searched

--- Add a ranker
---
--- @param name string
--- @param callback fun(results: { result: Result, providers: { provider: string, ranking: number }[] }[], options: table<string, string|number|boolean>): [Result]
function add_ranker(name, callback) end
