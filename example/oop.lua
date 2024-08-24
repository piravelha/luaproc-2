Animal = {}
Animal.__index = Animal
function Animal.new(...)
	local self = setmetatable({}, Animal)
	if self.init then
		self:init(...)
	end
	return self
end
Animal.init = function(self)
	self.name = "Unnamed"
end
Animal.set_name = function(self, name)
	self.name = name
end
Animal.speak = function(self)
	print(self.name .. " makes a sound")
end
local cat = Animal.new()
cat:set_name("Whiskers")
cat:speak()
