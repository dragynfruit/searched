-- -- JSON engine for Searched
-- -- Licensed MIT.
-- -- (c) 2024 Dragynfruit

local function get_key(data, key)
	if key == nil then
		return data
	end

	local keys = {}
	for k in string.gmatch(key, '([^/]+)') do
		table.insert(keys, k)
	end

	local value = data
	for _, k in ipairs(keys) do
		if value[k] == nil then
			local index = tonumber(k)
			if index then
				value = value[index]
			else
				value = nil
			end
		else
			value = value[k]
		end

		if value == nil then
			break
		end
	end

	return value
end

add_engine('json', function(client, query, opts)
	local url = Url.from_template(tostring(opts.url), {
		query = query.query,
		page = tostring(query.page),
	}):string()

	local res = client:get(url, {})
	local data = parse_json(res)

	if opts.results_key then
		data = get_key(data, opts.results_key)
	end

	--- @type [Result]
	local results = {}
	for i = 1, #data do
		if data[i] ~= nil then
			local result = data[i]
			local result_url = get_key(result, opts.url_key)

			if result_url ~= nil then
				if opts.url_prefix then
					result_url = opts.url_prefix .. result_url
				end

				results[i] = {
					title = get_key(result, opts.title_key),
					url = result_url,
					general = {
						snippet = get_key(result, opts.snippet_key),
					},
				}
			end
		end
	end

	return results
end)
