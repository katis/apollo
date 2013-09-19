function add(a, b)
	return a + b
end

function noret(a, b)
	a = b
end

function concat(a, b)
	return a .. b
end

function reverseplus(arr, p)
	local newArr = {}
	local len = table.getn(arr)

	for i, v in ipairs(arr) do
		newArr[len - i] = v + p
	end

	return newArr
end

function swapper(arr)
	local newArr = {}

	for k, v in pairs(arr) do
		newArr[v] = k
	end

	return newArr
end

function call(f, o)
	f(o, 123)
end

function retself(s)
	return s
end
