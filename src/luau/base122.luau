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

local kShortened = 0b111 -- uses the illegal index to signify the last two-byte char encodes <= 7 bits.

local function getBufferSize(input: string)
	-- 87.5% efficiency
	-- local decoded = buffer.create(math.floor(string.len(input) * 7 / 8))
	local decodedIndex = 0
	local bitOfByte = 0

	local function push7(byte: number)
		byte = bit32.lshift(byte, 1)
		bitOfByte += 7
		if bitOfByte >= 8 then
			decodedIndex += 1
			bitOfByte -= 8
		end
	end

	for _, character in utf8.codes(input) do
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

	return decodedIndex
end

-- Returns the decoded buffer and the actual, usable length of the buffer.
local function decode(input: string): (buffer, number)
	-- 87.5% efficiency
	-- math.floor(string.len(input) * 7 / 8)?
	local decoded = buffer.create(getBufferSize(input))
	local decodedIndex = 0
	local currentByte = 0
	local bitOfByte = 0

	local function push7(byte: number)
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

	for _, character in utf8.codes(input) do
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

local decoded, actualLength = decode(
	"\194\128\32\32\24\16\198\134\3\66\1\16\80\44\24\202\135\3\98\1\8\72\38\20\199\133\66\113\64\100\52\27\14\7\35\97\121\195\130\206\145\73\4\82\49\28\80\41\21\199\165\66\105\56\94\48\24\76\70\51\33\84\108\55\28\14\39\35\89\112\122\62\31\80\8\20\18\202\136\69\35\17\105\4\74\41\22\76\38\83\73\117\2\69\36\83\42\21\42\101\58\97\50\90\45\87\11\85\114\125\64\97\49\24\108\70\43\25\78\104\52\90\77\54\99\53\211\175\56\28\46\39\27\81\106\118\59\94\15\23\83\109\120\125\63\31\112\8\12\198\135\4\66\97\80\120\68\38\21\11\70\35\49\104\124\66\35\18\73\101\18\89\52\94\49\25\77\38\115\73\108\122\63\32\80\104\84\58\37\22\77\39\84\42\53\42\93\50\91\46\87\108\22\27\21\78\105\53\91\45\119\11\77\106\119\60\94\111\87\124\6\7\5\67\98\49\56\108\62\35\19\74\101\115\25\211\182\63\33\81\105\52\122\77\46\91\47\88\108\118\91\61\102\119\61\95\112\56\60\46\31\19\75\102\115\122\29\30\87\47\89\109\119\59\126\15\15\75\103\116\122\125\94\127\71\103\117\123\126\63\63\111\120"
)

for i = 0, actualLength - 1 do
	assert(i == buffer.readu8(decoded, i), "failed for i: " .. i)
end

return decode
