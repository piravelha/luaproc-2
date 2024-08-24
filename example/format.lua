local thing = "pre processors"
print(("I love %s!"):format(thing))
local x = (function()
	local x = 5
	local y = x + 1
	return y - 1
end)()
print(("x is: %d"):format(x))
