local person = { name = "Ian", age = 15 }
local name, age
do
	local o = {}
	local fields = { "name", "age" }
	for k, v in pairs(person) do
		for i, k2 in pairs(fields) do
			if k == k2 then
				o[i] = v
			end
		end
	end
	name, age = unpack(o)
end
print(name, age)
