-- -- JSON engine for Searched
-- -- Licensed MIT.
-- -- (c) 2024 Dragynfruit

local function dump(o)
	if type(o) == 'table' then
	   local s = '{ '
	   for k,v in pairs(o) do
		  if type(k) ~= 'number' then k = '"'..k..'"' end
		  s = s .. '['..k..'] = ' .. dump(v) .. ','
	   end
	   return s .. '} '
	else
	   return tostring(o)
	end
end

add_engine("json_engine", function(client, _, url, opts)
	if url ~= nil then
		local res = client:get(url, {})
		local data = parse_json(res)

		if opts.results_key then
			data = data[opts.results_key]
		end

		local results = {}
		for i, _ in ipairs(data) do
			results[i] = {
				title = data[i][opts.title_key],
				url = data[i][opts.url_key],
				snippet = data[i][opts.snippet_key],
			}
		end

		return results
	else
		return {}
	end
end)
