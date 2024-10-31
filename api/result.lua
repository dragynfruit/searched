--- @meta searched

--- @class Result
--- A search result
---
--- @field public url string
--- @field public title string
--- @field public general? GeneralResult
--- @field public forum? ForumResult
--- @field public image? ImageResult
Result = {}

--- @class GeneralResult
---
--- @field public snippet string
GeneralResult = {}

--- @class ForumResult
---
--- @field public poster_image? string
--- @field public poster_username string
--- @field public poster_url? string
--- @field public tags? [string]
ForumResult = {}

--- @class ImageResult
---
--- @field public preview_url string
--- @field public full_size_url string
ImageResult = {}
