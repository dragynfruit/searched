--local url = Url.parse('https://youtube.com/watch?v=dUqnciU7Mc')

local Services = {}

--- @param url Url
function Services.reddit(url)
end

Services['reddit'](Url.parse(''))

local a = {
	['reddit.com'] = 'reddit',
	['www.reddit.com'] = 'reddit',
	['old.reddit.com'] = 'reddit',
}
Services[a['reddit.com']]()

Services['youtube.com'] = function()
	-- replace with invidious
end
Services['www.youtube.com'] = Services['youtube.com']
