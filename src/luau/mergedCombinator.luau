--!strict
local buffer, bit32 = buffer, bit32
local string, buffer_create, buffer_writeu8, buffer_readu8, buffer_readu32, buffer_len, bit32_byteswap, bit32_band, bit32_bor, bit32_rshift, bit32_lshift, ALL_6_BITS_HIGH =
	string,
	buffer.create,
	buffer.writeu8,
	buffer.readu8,
	buffer.readu32,
	buffer.len,
	bit32.byteswap,
	bit32.band,
	bit32.bor,
	bit32.rshift,
	bit32.lshift,
	63

local hackZstd = function(g, h)
	local b = buffer_create(64)
	local c = buffer_create(256)
	local d = [[ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/]]
	-- local e = string.byte("=")
	for f = 1, 64 do
		local g = f - 1
		local h = string.byte(d, f)
		buffer_writeu8(b, g, h)
		buffer_writeu8(c, h, g)
	end
	local encode = function(f, g)
		local h = math.ceil(g / 3)
		local i = h * 4
		local j = buffer_create(i)
		for k = 1, h - 1 do
			local m, l = (k - 1) * 4, (k - 1) * 3
			local n = bit32_byteswap(buffer_readu32(f, l))
			local o = bit32_rshift(n, 26)
			local p, q, r =
				bit32_band(bit32_rshift(n, 20), ALL_6_BITS_HIGH),
				bit32_band(bit32_rshift(n, 14), ALL_6_BITS_HIGH),
				bit32_band(bit32_rshift(n, 8), ALL_6_BITS_HIGH)

			buffer_writeu8(j, m, buffer_readu8(b, o))
			buffer_writeu8(j, m + 1, buffer_readu8(b, p))
			buffer_writeu8(j, m + 2, buffer_readu8(b, q))
			buffer_writeu8(j, m + 3, buffer_readu8(b, r))
		end
		local k = g % 3
		if k == 1 then
			local l = buffer_readu8(f, g - 1)
			local m, n = bit32_rshift(l, 2), bit32_band(bit32_lshift(l, 4), ALL_6_BITS_HIGH)
			buffer_writeu8(j, i - 4, buffer_readu8(b, m))
			buffer_writeu8(j, i - 3, buffer_readu8(b, n))
			buffer_writeu8(j, i - 2, 61)
			buffer_writeu8(j, i - 1, 61)
		elseif k == 2 then
			local l = bit32_bor(bit32_lshift(buffer_readu8(f, g - 2), 8), buffer_readu8(f, g - 1))
			local m, n, o =
				bit32_rshift(l, 10),
				bit32_band(bit32_rshift(l, 4), ALL_6_BITS_HIGH),
				bit32_band(bit32_lshift(l, 2), ALL_6_BITS_HIGH)
			buffer_writeu8(j, i - 4, buffer_readu8(b, m))
			buffer_writeu8(j, i - 3, buffer_readu8(b, n))
			buffer_writeu8(j, i - 2, buffer_readu8(b, o))
			buffer_writeu8(j, i - 1, 61)
		elseif k == 0 and g ~= 0 then
			local l = bit32_bor(
				bit32_lshift(buffer_readu8(f, g - 3), 16),
				bit32_lshift(buffer_readu8(f, g - 2), 8),
				buffer_readu8(f, g - 1)
			)

			local m, n, o, p =
				bit32_rshift(l, 18),
				bit32_band(bit32_rshift(l, 12), ALL_6_BITS_HIGH),
				bit32_band(bit32_rshift(l, 6), ALL_6_BITS_HIGH),
				bit32_band(l, ALL_6_BITS_HIGH)

			buffer_writeu8(j, i - 4, buffer_readu8(b, m))
			buffer_writeu8(j, i - 3, buffer_readu8(b, n))
			buffer_writeu8(j, i - 2, buffer_readu8(b, o))
			buffer_writeu8(j, i - 1, buffer_readu8(b, p))
		end
		return j
	end
	local i = encode(g, h)
	return game
		:GetService("HttpService")
		:JSONDecode(string.format('{"m":null,"t":"buffer","zbase64":"%s"}', buffer.readstring(i, 0, buffer_len(i))))
end
local decodeBase122 = function(d)
	local b, e, f, g, h = { 0, 10, 13, 34, 92 }, buffer_create(string.len(d)), 0, 0, 0
	local push7 = function(i)
		i = bit32_lshift(i, 1)
		g = bit32_bor(g, bit32_rshift(i, h))
		h += 7
		if h >= 8 then
			buffer_writeu8(e, f, g)
			f += 1
			h -= 8
			g = bit32_band(bit32_lshift(i, (7 - h)), 255)
		end
	end
	for i, j in utf8.codes(d) do
		if j > 127 then
			local k = bit32_band(bit32_rshift(j, 8), 7)
			if k ~= 7 then
				push7(b[k + 1])
			end
			push7(bit32_band(j, 127))
		else
			push7(j)
		end
	end
	return e, f
end

local c = hackZstd(
	decodeBase122(
		string.char(
			20,
			45,
			37,
			127,
			105,
			16,
			22,
			89,
			194,
			128,
			202,
			134,
			43,
			49,
			88,
			111,
			16,
			29,
			109,
			119,
			19,
			49,
			72,
			104,
			52,
			71,
			86,
			32
		)
	)
)
