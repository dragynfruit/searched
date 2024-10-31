local agent = 'Algolia for JavaScript (4.24.0)%3B Browser (lite)'

add_engine('algolia', function(client, query, opts)
	assert(type(opts['application_id']) == 'string', 'application_id must be set')
	assert(type(opts['api_key']) == 'string', 'api_key must be set')
	assert(type(opts['distributed']) == 'boolean', 'distributed must be set')

	--
end)
