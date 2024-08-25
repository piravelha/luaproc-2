local Color = {}
do
	local fields = { "red", "green", "blue" }
	for i, field in pairs(fields) do
		Color[i] = field
		Color[field] = i
	end
end
print(Color.red, Color.green, Color.blue)
print(Color[1], Color[2], Color[3])
