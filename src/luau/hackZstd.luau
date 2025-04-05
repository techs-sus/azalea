--!native
--!optimize 2

local lookupValueToCharacter = buffer.create(64)
local lookupCharacterToValue = buffer.create(256)

local alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
local padding = string.byte("=")

for index = 1, 64 do
	local value = index - 1
	local character = string.byte(alphabet, index)

	buffer.writeu8(lookupValueToCharacter, value, character)
	buffer.writeu8(lookupCharacterToValue, character, value)
end

local function encode(input: buffer, inputLength: number?): buffer
	inputLength = inputLength or buffer.len(input)
	local inputChunks = math.ceil(inputLength / 3)

	local outputLength = inputChunks * 4
	local output = buffer.create(outputLength)

	-- Since we use readu32 and chunks are 3 bytes large, we can't read the last chunk here
	for chunkIndex = 1, inputChunks - 1 do
		local inputIndex = (chunkIndex - 1) * 3
		local outputIndex = (chunkIndex - 1) * 4

		local chunk = bit32.byteswap(buffer.readu32(input, inputIndex))

		-- 8 + 24 - (6 * index)
		local value1 = bit32.rshift(chunk, 26)
		local value2 = bit32.band(bit32.rshift(chunk, 20), 0b111111)
		local value3 = bit32.band(bit32.rshift(chunk, 14), 0b111111)
		local value4 = bit32.band(bit32.rshift(chunk, 8), 0b111111)

		buffer.writeu8(output, outputIndex, buffer.readu8(lookupValueToCharacter, value1))
		buffer.writeu8(output, outputIndex + 1, buffer.readu8(lookupValueToCharacter, value2))
		buffer.writeu8(output, outputIndex + 2, buffer.readu8(lookupValueToCharacter, value3))
		buffer.writeu8(output, outputIndex + 3, buffer.readu8(lookupValueToCharacter, value4))
	end

	local inputRemainder = inputLength % 3

	if inputRemainder == 1 then
		local chunk = buffer.readu8(input, inputLength - 1)

		local value1 = bit32.rshift(chunk, 2)
		local value2 = bit32.band(bit32.lshift(chunk, 4), 0b111111)

		buffer.writeu8(output, outputLength - 4, buffer.readu8(lookupValueToCharacter, value1))
		buffer.writeu8(output, outputLength - 3, buffer.readu8(lookupValueToCharacter, value2))
		buffer.writeu8(output, outputLength - 2, padding)
		buffer.writeu8(output, outputLength - 1, padding)
	elseif inputRemainder == 2 then
		local chunk =
			bit32.bor(bit32.lshift(buffer.readu8(input, inputLength - 2), 8), buffer.readu8(input, inputLength - 1))

		local value1 = bit32.rshift(chunk, 10)
		local value2 = bit32.band(bit32.rshift(chunk, 4), 0b111111)
		local value3 = bit32.band(bit32.lshift(chunk, 2), 0b111111)

		buffer.writeu8(output, outputLength - 4, buffer.readu8(lookupValueToCharacter, value1))
		buffer.writeu8(output, outputLength - 3, buffer.readu8(lookupValueToCharacter, value2))
		buffer.writeu8(output, outputLength - 2, buffer.readu8(lookupValueToCharacter, value3))
		buffer.writeu8(output, outputLength - 1, padding)
	elseif inputRemainder == 0 and inputLength ~= 0 then
		local chunk = bit32.bor(
			bit32.lshift(buffer.readu8(input, inputLength - 3), 16),
			bit32.lshift(buffer.readu8(input, inputLength - 2), 8),
			buffer.readu8(input, inputLength - 1)
		)

		local value1 = bit32.rshift(chunk, 18)
		local value2 = bit32.band(bit32.rshift(chunk, 12), 0b111111)
		local value3 = bit32.band(bit32.rshift(chunk, 6), 0b111111)
		local value4 = bit32.band(chunk, 0b111111)

		buffer.writeu8(output, outputLength - 4, buffer.readu8(lookupValueToCharacter, value1))
		buffer.writeu8(output, outputLength - 3, buffer.readu8(lookupValueToCharacter, value2))
		buffer.writeu8(output, outputLength - 2, buffer.readu8(lookupValueToCharacter, value3))
		buffer.writeu8(output, outputLength - 1, buffer.readu8(lookupValueToCharacter, value4))
	end

	return output
end

local HttpService = game:GetService("HttpService")

local function decodeZstd(buf: buffer, inputLength: number?): buffer
	inputLength = inputLength or buffer.len(buf)
	local b64 = encode(buf, inputLength)
	return HttpService:JSONDecode(
		string.format([[{"m":null,"t":"buffer","zbase64":"%s"}]], buffer.readstring(b64, 0, buffer.len(b64)))
	)
end

return decodeZstd
