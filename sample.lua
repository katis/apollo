function add(a, b)
	return a + b
end

function noret(a, b)
	a = b
end

function concat(a, b)
	return a .. b
end

function reverse(arr)
	local newArr = {}
	local len = table.getn(arr)

	for i, v in ipairs(arr) do
		newArr[len - i] = v
	end

	return newArr
end
