local ok, err = pcall(function()
	error("Oh no!")
end)
if not ok then
	local e = err
	print(e)
	print("Oh, its fine.")
end
