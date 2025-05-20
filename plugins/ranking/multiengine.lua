PROVIDER_WEIGHTS = {}

add_ranker('multiprovider', function(results, options)
	local weights = {}

	for i, res in ipairs(results) do
		for _, provider in ipairs(res.providers) do
			local provider_weight = PROVIDER_WEIGHTS[provider]

			if provider_weight == nil then
				provider_weight = 1.1
			end

			if weights[i] == nil then
				weights[i] = 1.0
			else
				weights[i] = weights[i] * provider_weight
			end
		end
	end

	return weights
end)

local table_concat = function (dst, src)
   for i = 1, #src, 1 do
      dst[#dst+1] = src[i]
   end
	 return dst
end

add_merger('multiprovider', function(results, options)
	for i = 1, #results, 1 do
		local max_weight = 1.1

		for j = #results, i+1, -1 do
			if results[i].url == results[j].url then
				local provider_weight = PROVIDER_WEIGHTS[results[i].providers[1]]

				if provider_weight ~= nil and provider_weight > max_weight then
					results[j].providers = table_concat(results[j].providers, results[i].providers)
					results[i] = results[j]
				end

				results[i].providers = table_concat(results[i].providers, results[j].providers)
				results[j].providers = {}
			end
		end
	end

	return results
end)
