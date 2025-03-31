--!native
--!optimize 2
--!strict

--[[
	Base122 - A space efficent alternative to base-64.
	>>> https://blog.kevinalbs.com/base122
]]

-- to create Base123, we comment out ampersand being illegal...
local kIllegals = {
	0, -- null
	10, -- newline
	13, -- carriage return
	34, -- double quote
	-- 38, -- SKIP! ampersand, without this being illegal we have base123
	92, -- backslash
}

local kShortened = 0b111 -- Uses the illegal index to signify the last two-byte char encodes <= 7 bits.

-- Returns the decoded buffer and the length of the buffer.
local function decode(inputString: string): (buffer, number)
	-- 87.5% efficiency
	local decoded = buffer.create(string.len(inputString))
	local decodedIndex = 0
	local currentByte = 0
	local bitOfByte = 0

	local function push7(byte)
		byte = bit32.lshift(byte, 1)
		currentByte = bit32.bor(currentByte, bit32.rshift(byte, bitOfByte))
		bitOfByte += 7
		if bitOfByte >= 8 then
			buffer.writeu8(decoded, decodedIndex, currentByte)
			decodedIndex += 1
			bitOfByte -= 8
			currentByte = bit32.band(bit32.lshift(byte, (7 - bitOfByte)), 255)
		end
	end

	for _, character in utf8.codes(inputString) do
		if character > 127 then
			local illegalIndex = bit32.band(bit32.rshift(character, 8), 7)
			if illegalIndex ~= kShortened then
				push7(kIllegals[illegalIndex + 1])
			end
			push7(bit32.band(character, 127))
		else
			push7(character)
		end
	end

	return decoded, decodedIndex
end

local decoded, actualLength = decode(string.char(194, 130, 65, 82, 18, 112))
print(buffer.readstring(decoded, 0, actualLength))
