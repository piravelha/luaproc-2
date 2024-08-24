local _ok_, _err_ = pcall(function()
	error("Oh no!")
end)
if not _ok_ then
	local e = _err_
	print(e)
	print("Oh, its fine.")
end
