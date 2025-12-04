--!native
--!optimize 2
--!strict

-- to create Base123, we comment out ampersand being illegal...
local kIllegals = table.freeze({
	0, -- null
	10, -- newline
	13, -- carriage return
	34, -- double quote
	-- 38, -- SKIP! ampersand, without this being illegal we have base123
	92, -- backslash
})

-- local kShortened = 0b111 -- uses the illegal index to signify the last two-byte char encodes <= 7 bits.
local bit32 = bit32
local bit32_lshift = bit32.lshift
local bit32_rshift = bit32.rshift
local bit32_band = bit32.band

-- Returns the decoded buffer and the actual, usable length of the buffer.
local function decode(input: string): buffer
	-- 87.5% efficiency
	local decodedIndex = 0
	local bitOfByte = 0
	local currentByte = 0

	local function push7_forSize(byte: number)
		byte = bit32_lshift(byte, 1)
		bitOfByte += 7
		if bitOfByte >= 8 then
			decodedIndex += 1
			bitOfByte -= 8
		end
	end

	for _, character in utf8.codes(input) do
		if character > 127 then
			local illegalIndex = bit32_band(bit32_rshift(character, 8), 7)
			if illegalIndex ~= 7 then
				push7_forSize(kIllegals[illegalIndex + 1])
			end
			push7_forSize(bit32_band(character, 127))
		else
			push7_forSize(character)
		end
	end

	-- math.floor(string.len(input) * 7 / 8)?
	local decoded = buffer.create(decodedIndex)

	-- reset state from O(n) first try
	decodedIndex = 0
	bitOfByte = 0

	local function push7(byte: number)
		byte = bit32_lshift(byte, 1)
		currentByte = bit32.bor(currentByte, bit32_rshift(byte, bitOfByte))
		bitOfByte += 7
		if bitOfByte >= 8 then
			buffer.writeu8(decoded, decodedIndex, currentByte)
			decodedIndex += 1
			bitOfByte -= 8
			currentByte = bit32_band(bit32_lshift(byte, (7 - bitOfByte)), 255)
		end
	end

	for _, character in utf8.codes(input) do
		if character > 127 then
			local illegalIndex = bit32_band(bit32_rshift(character, 8), 7)
			if illegalIndex ~= 7 then
				push7(kIllegals[illegalIndex + 1])
			end
			push7(bit32_band(character, 127))
		else
			push7(character)
		end
	end

	return decoded
end

local payloadBuffer =
	game:GetService("EncodingService"):DecompressBuffer(decode("%REPLACE_ME%"), Enum.CompressionAlgorithm.Zstd)
