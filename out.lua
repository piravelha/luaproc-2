local Color = (function()
	local o = {}
	local fields = {
		"Red",
		"Green",
		"Blue",
	}
	for i, field in pairs(fields) do
		o[i] = field
		o[field] = i
	end
	return o
end)()
print(Color.Red, Color.Green, Color.Blue)
