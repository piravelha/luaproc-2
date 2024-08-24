function do_something(x)
	if not (type(x) == "number") then
		return nil
	end
	return x * x
end
function do_something_else(x)
	if not (type(x) == "string") then
		return nil
	end
	return x .. "!"
end
function main(input)
	local x = do_something(input)
	if x == nil then
		return nil
	end
	local s = tostring(x)
	local y = do_something_else(s)
	if y == nil then
		return nil
	end
	return y
end
print(main("a"))
