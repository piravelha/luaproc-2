local f
if (function()
	f = 1 + 2
	return f
end)() == f then
	print(f)
end
