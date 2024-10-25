function dump(o)
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

add_search_provider('wikipedia', 'wiki', function (query)
	local res = get('https://en.wikipedia.org/w/api.php?action=opensearch&format=json&limit=10&namespace=0&search='..query.query, {})

	local data = parse_json(res)

	local results = {}
	if data[2] ~= nil then
		for i, _ in ipairs(data[2]) do
			if data[3] ~= nil then
				results[i] = {
					title = data[2][i],
					url = data[3][i],
				}
			end
		end
	end

	return results
end)
