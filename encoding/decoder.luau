--!native
--!optimize 2

-- WARNING: This file is @generated by azalea. Do not edit.

-- @generated
local TYPE_ID = table.freeze({
	String = 0,
	Attributes = 1,
	Axes = 2,
	Bool = 3,
	BrickColor = 4,
	CFrame = 5,
	Color3 = 6,
	Color3uint8 = 7,
	ColorSequence = 8,
	Enum = 9,
	Faces = 10,
	Float32 = 11,
	Float64 = 12,
	Int32 = 13,
	MaterialColors = 14,
	NumberRange = 15,
	NumberSequence = 16,
	None = 17,
	DefaultPhysicalProperties = 18,
	CustomPhysicalProperties = 19,
	Ray = 20,
	Rect = 21,
	Ref = 22,
	Region3 = 23,
	Region3int16 = 24,
	SecurityCapabilities = 25,
	BinaryString = 26,
	Tags = 27,
	UDim = 28,
	UDim2 = 29,
	Vector2 = 30,
	Vector2int16 = 31,
	Vector3 = 32,
	Vector3int16 = 33,
	Font = 34,
})
local CFRAME_ID_LOOKUP_TABLE = table.freeze({
	[0x02] = CFrame.fromEulerAnglesYXZ(0, 0, 0),
	[0x03] = CFrame.fromEulerAnglesYXZ(math.rad(90), 0, 0),
	[0x05] = CFrame.fromEulerAnglesYXZ(0, math.rad(180), math.rad(180)),
	[0x06] = CFrame.fromEulerAnglesYXZ(math.rad(-90), 0, 0),
	[0x07] = CFrame.fromEulerAnglesYXZ(0, math.rad(180), math.rad(90)),
	[0x09] = CFrame.fromEulerAnglesYXZ(0, math.rad(90), math.rad(90)),
	[0x0a] = CFrame.fromEulerAnglesYXZ(0, 0, math.rad(90)),
	[0x0c] = CFrame.fromEulerAnglesYXZ(0, math.rad(-90), math.rad(90)),
	[0x0d] = CFrame.fromEulerAnglesYXZ(math.rad(-90), math.rad(-90), 0),
	[0x0e] = CFrame.fromEulerAnglesYXZ(math.rad(0), math.rad(-90), 0),
	[0x10] = CFrame.fromEulerAnglesYXZ(math.rad(90), math.rad(-90), 0),
	[0x11] = CFrame.fromEulerAnglesYXZ(math.rad(0), math.rad(90), 180),

	[0x14] = CFrame.fromEulerAnglesYXZ(0, math.rad(180), 0),
	[0x15] = CFrame.fromEulerAnglesYXZ(math.rad(-90), math.rad(-180), 0),
	[0x17] = CFrame.fromEulerAnglesYXZ(0, 0, math.rad(180)),
	[0x18] = CFrame.fromEulerAnglesYXZ(math.rad(90), math.rad(180), 0),
	[0x19] = CFrame.fromEulerAnglesYXZ(0, 0, math.rad(-90)),
	[0x1b] = CFrame.fromEulerAnglesYXZ(0, math.rad(-90), math.rad(-90)),
	[0x1c] = CFrame.fromEulerAnglesYXZ(0, math.rad(-180), math.rad(-90)),
	[0x1e] = CFrame.fromEulerAnglesYXZ(0, math.rad(90), math.rad(-90)),
	[0x1f] = CFrame.fromEulerAnglesYXZ(math.rad(90), math.rad(90), 0),
	[0x20] = CFrame.fromEulerAnglesYXZ(0, math.rad(90), 0),
	[0x22] = CFrame.fromEulerAnglesYXZ(math.rad(-90), math.rad(90), 0),
	[0x23] = CFrame.fromEulerAnglesYXZ(0, math.rad(-90), math.rad(180)),
})
local NewScript: (code: string, parent: Instance?) -> Script = NewScript
	or function(code, parent)
		local script = Instance.new("Script")
		script.Source = code
		script.Parent = parent

		return script
	end
local NewLocalScript: (code: string, parent: Instance?) -> LocalScript = NewLocalScript
	or function(code, parent)
		local script = Instance.new("LocalScript")
		script.Source = code
		script.Parent = parent

		return script
	end
local NewModuleScript: (code: string, parent: Instance?) -> ModuleScript = NewModuleScript
	or function(code, parent)
		local script = Instance.new("ModuleScript")
		script.Source = code
		script.Parent = parent

		return script
	end

local function decode(payloadBuffer: buffer)
	local nilParentedInstance = Instance.new("Folder", nil)

	local loc = 0
	local VARIANT_DECODER: { [number]: () -> any } = nil
	local nextVariant

	local function nextNullstring(): string
		local start = loc
		while true do
			loc += 1
			if buffer.readu8(payloadBuffer, loc) == 0 then
				-- ensure we skip null byte
				loc += 1
				break
			end
		end

		-- ensure we skip null byte in decoded string
		return buffer.readstring(payloadBuffer, start, loc - start - 1)
	end

	local function nextUnsignedInteger(): number
		local result = 0
		local shift = 0
		local byte

		repeat
			byte = buffer.readu8(payloadBuffer, loc)
			loc += 1

			result = bit32.bor(result, bit32.lshift(bit32.band(byte, 0x7F), shift))
			shift = shift + 7

			if shift >= 32 and byte >= 0x80 then
				error("leb128 overflow (exceeded 32 bits)")
			end
		until bit32.band(byte, 0x80) == 0

		return result
	end

	-- @generated
	VARIANT_DECODER = table.freeze({
		[TYPE_ID.String] = function()
			local stringLength = nextUnsignedInteger()
			loc += stringLength
			return buffer.readstring(payloadBuffer, loc - stringLength, stringLength)
		end,
		[TYPE_ID.Attributes] = function()
			local attributesLength = nextUnsignedInteger()
			local attributeMap: { [string]: any } = {}

			while attributesLength > 0 do
				local attributeName = nextNullstring()
				attributeMap[attributeName] = nextVariant()
				-- print(attributeName, #attributeName, attributeMap[attributeName])

				attributesLength -= 1
			end

			return attributeMap
		end,
		[TYPE_ID.Axes] = function()
			local byte = buffer.readu8(payloadBuffer, loc)
			loc += 1
			-- bitflags
			-- const X = 1;
			-- const Y = 2;
			-- const Z = 4;
			local x = if bit32.extract(byte, 0, 1) == 1 then Enum.Axis.X else nil
			local y = if bit32.extract(byte, 1, 1) == 1 then Enum.Axis.Y else nil
			local z = if bit32.extract(byte, 2, 1) == 1 then Enum.Axis.Z else nil

			return Axes.new(x, y, z)
		end,
		[TYPE_ID.Bool] = function()
			local bool = buffer.readu8(payloadBuffer, loc)
			loc += 1
			return bool == 1
		end,
		[TYPE_ID.BrickColor] = function()
			return BrickColor.new(nextNullstring() :: any)
		end,
		[TYPE_ID.CFrame] = function()
			local id = buffer.readu8(payloadBuffer, loc)
			loc += 1

			if id == 0 then
				-- all data is encoded
				local xx, xy, xz, yx, yy, yz, zx, zy, zz, positionX, positionY, positionZ =
					buffer.readf32(payloadBuffer, loc),
					buffer.readf32(payloadBuffer, loc + 4),
					buffer.readf32(payloadBuffer, loc + 8),
					buffer.readf32(payloadBuffer, loc + 12),
					buffer.readf32(payloadBuffer, loc + 16),
					buffer.readf32(payloadBuffer, loc + 20),
					buffer.readf32(payloadBuffer, loc + 24),
					buffer.readf32(payloadBuffer, loc + 28),
					buffer.readf32(payloadBuffer, loc + 32),
					buffer.readf32(payloadBuffer, loc + 36),
					buffer.readf32(payloadBuffer, loc + 40),
					buffer.readf32(payloadBuffer, loc + 44)

				loc += 48

				return CFrame.new(positionX, positionY, positionZ, xx, xy, xz, yx, yy, yz, zx, zy, zz)
			else
				local positionX, positionY, positionZ =
					buffer.readf32(payloadBuffer, loc),
					buffer.readf32(payloadBuffer, loc + 4),
					buffer.readf32(payloadBuffer, loc + 8)
				loc += 12

				return CFrame.new(positionX, positionY, positionZ) * CFRAME_ID_LOOKUP_TABLE[id]
			end
		end,
		[TYPE_ID.Color3] = function()
			local r, g, b =
				buffer.readf32(payloadBuffer, loc),
				buffer.readf32(payloadBuffer, loc + 4),
				buffer.readf32(payloadBuffer, loc + 8)

			loc += 12
			return Color3.new(r, g, b)
		end,
		[TYPE_ID.Color3uint8] = function()
			local r, g, b =
				buffer.readu8(payloadBuffer, loc), buffer.readu8(payloadBuffer, loc + 1), buffer.readu8(payloadBuffer, loc + 2)
			loc += 3

			return Color3.fromRGB(r, g, b)
		end,
		[TYPE_ID.ColorSequence] = function()
			local keypointsLength = nextUnsignedInteger()
			local keypoints: { ColorSequenceKeypoint } = {}

			while keypointsLength > 0 do
				local time, r, g, b =
					buffer.readf32(payloadBuffer, loc),
					buffer.readf32(payloadBuffer, loc + 4),
					buffer.readf32(payloadBuffer, loc + 8),
					buffer.readf32(payloadBuffer, loc + 12)

				table.insert(keypoints, ColorSequenceKeypoint.new(time, Color3.new(r, g, b)))

				loc += 16
				keypointsLength -= 1
			end

			return ColorSequence.new(keypoints)
		end,
		[TYPE_ID.Enum] = function()
			return nextUnsignedInteger()
		end,
		[TYPE_ID.Faces] = function()
			local byte = buffer.readu8(payloadBuffer, loc)
			loc += 1
			-- bitflags
			-- const RIGHT = 1;
			-- const TOP = 2;
			-- const BACK = 4;
			-- const LEFT = 8;
			-- const BOTTOM = 16;
			-- const FRONT = 32;
			local right = if bit32.extract(byte, 0, 1) == 1 then Enum.NormalId.Right else nil
			local top = if bit32.extract(byte, 1, 1) == 1 then Enum.NormalId.Top else nil
			local back = if bit32.extract(byte, 2, 1) == 1 then Enum.NormalId.Back else nil
			local left = if bit32.extract(byte, 3, 1) == 1 then Enum.NormalId.Left else nil
			local bottom = if bit32.extract(byte, 4, 1) == 1 then Enum.NormalId.Bottom else nil
			local front = if bit32.extract(byte, 5, 1) == 1 then Enum.NormalId.Front else nil

			return Faces.new(right, top, back, left, bottom, front)
		end,
		[TYPE_ID.Float32] = function()
			local float = buffer.readf32(payloadBuffer, loc)
			loc += 4
			return float
		end,
		[TYPE_ID.Float64] = function()
			local float = buffer.readf64(payloadBuffer, loc)
			loc += 8
			return float
		end,
		[TYPE_ID.Int32] = function()
			local int = buffer.readi32(payloadBuffer, loc)
			loc += 4
			return int
		end,
		[TYPE_ID.MaterialColors] = function()
			-- SKIP: Terrain.MaterialColors is not writable by scripts + we could use :SetMaterialColor(), but theres no reason to support that
			loc += 69
		end,
		[TYPE_ID.NumberRange] = function()
			local min, max = buffer.readf32(payloadBuffer, loc), buffer.readf32(payloadBuffer, loc + 4)
			loc += 8

			return NumberRange.new(min, max)
		end,
		[TYPE_ID.NumberSequence] = function()
			local numberSequenceLength = nextUnsignedInteger()
			local keypoints: { NumberSequenceKeypoint } = {}

			while numberSequenceLength > 0 do
				local envelope, time, value =
					buffer.readf32(payloadBuffer, loc),
					buffer.readf32(payloadBuffer, loc + 4),
					buffer.readf32(payloadBuffer, loc + 8)
				loc += 12

				table.insert(keypoints, NumberSequenceKeypoint.new(time, value, envelope))

				numberSequenceLength -= 1
			end

			return NumberSequence.new(keypoints)
		end,
		[TYPE_ID.None] = function()
			return nil
		end,
		[TYPE_ID.DefaultPhysicalProperties] = function()
			-- SKIP: DefaultPhysicalProperties does not translate to Luau without a material
			return nil
		end,
		[TYPE_ID.CustomPhysicalProperties] = function()
			local density, elasticity, elasticityWeight, friction, frictionWeight =
				buffer.readf32(payloadBuffer, loc),
				buffer.readf32(payloadBuffer, loc + 4),
				buffer.readf32(payloadBuffer, loc + 8),
				buffer.readf32(payloadBuffer, loc + 12),
				buffer.readf32(payloadBuffer, loc + 16)

			loc += 20
			return PhysicalProperties.new(density, friction, elasticity, frictionWeight, elasticityWeight)
		end,
		[TYPE_ID.Ray] = function()
			local directionX, directionY, directionZ, originX, originY, originZ =
				buffer.readf32(payloadBuffer, loc),
				buffer.readf32(payloadBuffer, loc + 4),
				buffer.readf32(payloadBuffer, loc + 8),
				buffer.readf32(payloadBuffer, loc + 12),
				buffer.readf32(payloadBuffer, loc + 16),
				buffer.readf32(payloadBuffer, loc + 20)

			loc += 24

			return Ray.new(Vector3.new(originX, originY, originZ), Vector3.new(directionX, directionY, directionZ))
		end,
		[TYPE_ID.Rect] = function()
			local minX, minY, maxX, maxY =
				buffer.readf32(payloadBuffer, loc),
				buffer.readf32(payloadBuffer, loc + 4),
				buffer.readf32(payloadBuffer, loc + 8),
				buffer.readf32(payloadBuffer, loc + 12)

			loc += 16

			return Rect.new(minX, minY, maxX, maxY)
		end,
		[TYPE_ID.Ref] = function()
			return nextUnsignedInteger()
		end,
		[TYPE_ID.Region3] = function()
			-- We cannot test this: there are no properties to test this for.
			local minX, minY, minZ, maxX, maxY, maxZ =
				buffer.readf32(payloadBuffer, loc),
				buffer.readf32(payloadBuffer, loc + 4),
				buffer.readf32(payloadBuffer, loc + 8),
				buffer.readf32(payloadBuffer, loc + 12),
				buffer.readf32(payloadBuffer, loc + 16),
				buffer.readf32(payloadBuffer, loc + 20)

			loc += 24

			return Region3.new(Vector3.new(minX, minY, minZ), Vector3.new(maxX, maxY, maxZ))
		end,
		[TYPE_ID.Region3int16] = function()
			-- Tests cannot be implemented for the same reason as Region3
			local minX, minY, minZ, maxX, maxY, maxZ =
				buffer.readi16(payloadBuffer, loc),
				buffer.readi16(payloadBuffer, loc + 2),
				buffer.readi16(payloadBuffer, loc + 4),
				buffer.readi16(payloadBuffer, loc + 6),
				buffer.readi16(payloadBuffer, loc + 8),
				buffer.readi16(payloadBuffer, loc + 10)

			loc += 12
			return Region3int16.new(Vector3int16.new(minX, minY, minZ), Vector3int16.new(maxX, maxY, maxZ))
		end,
		[TYPE_ID.SecurityCapabilities] = function()
			-- SKIP: SecurityCapabilities is not writable in scripts
			loc += 8
		end,
		[TYPE_ID.BinaryString] = function()
			local stringLength = nextUnsignedInteger()
			loc += stringLength
			return buffer.readstring(payloadBuffer, loc - stringLength, stringLength)
		end,
		[TYPE_ID.Tags] = function()
			-- length of encoded array
			local tagsLength = nextUnsignedInteger()
			local tags = {}

			while tagsLength > 0 do
				local tag = nextNullstring()
				table.insert(tags, tag)

				tagsLength -= 1
			end

			return tags
		end,
		[TYPE_ID.UDim] = function()
			local offset, scale = buffer.readi32(payloadBuffer, loc), buffer.readf32(payloadBuffer, loc + 4)
			loc += 8

			return UDim.new(scale, offset)
		end,
		[TYPE_ID.UDim2] = function()
			local xOffset, yOffset, xScale, yScale =
				buffer.readi32(payloadBuffer, loc),
				buffer.readi32(payloadBuffer, loc + 4),
				buffer.readf32(payloadBuffer, loc + 8),
				buffer.readf32(payloadBuffer, loc + 12)

			loc += 16

			return UDim2.new(xScale, xOffset, yScale, yOffset)
		end,
		[TYPE_ID.Vector2] = function()
			local x, y = buffer.readf32(payloadBuffer, loc), buffer.readf32(payloadBuffer, loc + 4)
			loc += 8

			return Vector2.new(x, y)
		end,
		[TYPE_ID.Vector2int16] = function()
			local x, y = buffer.readi16(payloadBuffer, loc), buffer.readi16(payloadBuffer, loc + 2)
			loc += 4

			return Vector2int16.new(x, y)
		end,
		[TYPE_ID.Vector3] = function()
			local x, y, z =
				buffer.readf32(payloadBuffer, loc),
				buffer.readf32(payloadBuffer, loc + 4),
				buffer.readf32(payloadBuffer, loc + 8)
			loc += 12

			return Vector3.new(x, y, z)
		end,
		[TYPE_ID.Vector3int16] = function()
			local x, y, z =
				buffer.readi16(payloadBuffer, loc),
				buffer.readi16(payloadBuffer, loc + 2),
				buffer.readi16(payloadBuffer, loc + 4)
			loc += 6

			return Vector3int16.new(x, y, z)
		end,
		[TYPE_ID.Font] = function()
			local family = nextNullstring()
			local weight = buffer.readu16(payloadBuffer, loc)
			loc += 2
			local style = buffer.readu8(payloadBuffer, loc)
			loc += 1

			local weightEnum: Enum.FontWeight

			if weight == 100 then
				weightEnum = Enum.FontWeight.Thin
			elseif weight == 200 then
				weightEnum = Enum.FontWeight.ExtraLight
			elseif weight == 300 then
				weightEnum = Enum.FontWeight.Light
			elseif weight == 400 then
				weightEnum = Enum.FontWeight.Regular
			elseif weight == 500 then
				weightEnum = Enum.FontWeight.Medium
			elseif weight == 600 then
				weightEnum = Enum.FontWeight.SemiBold
			elseif weight == 700 then
				weightEnum = Enum.FontWeight.Bold
			elseif weight == 800 then
				weightEnum = Enum.FontWeight.ExtraBold
			elseif weight == 900 then
				weightEnum = Enum.FontWeight.Heavy
			else
				error(`font weight {weight} is not supported or is invalid`)
			end

			local styleEnum: Enum.FontStyle

			if style == 0 then
				styleEnum = Enum.FontStyle.Normal
			elseif style == 1 then
				styleEnum = Enum.FontStyle.Italic
			else
				error(`font style {style} is not supported or is invalid`)
			end

			return Font.new(family, weightEnum, styleEnum)
		end,
	})

	function nextVariant(expectedTypeIds: { number }?)
		-- 1. read type id
		-- 2. loc++;
		-- 3. call type id handler (which uses loc)

		local typeId = buffer.readu8(payloadBuffer, loc)
		loc += 1

		if expectedTypeIds and not table.find(expectedTypeIds, typeId) then
			error(`expected type id inside of array {table.concat(expectedTypeIds, ", ")}, got {typeId}`)
		end

		return assert(VARIANT_DECODER[typeId], "no variant decoder for type id " .. typeId)()
	end

	type Ref = number

	local rootReferent: Ref?
	local referentTree: { [string]: Instance } = {}

	-- late properties must be applied after the entire tree is decoded
	-- index signature: latePropertyMap[referent][propertyName] = propertyValue
	local latePropertiesMap: { [Ref]: { [string]: string } } = {}

	local function decodeInstance()
		local name: string = nextVariant({ TYPE_ID.String })
		local className: string = nextNullstring()
		local instanceReferent: Ref = nextVariant({ TYPE_ID.Ref })
		local parentReferent: Ref? = nextVariant({ TYPE_ID.Ref, TYPE_ID.None })

		local propertiesLength = buffer.readu16(payloadBuffer, loc)
		local propertiesMap: { [string]: any } = {}
		loc += 2

		while propertiesLength > 0 do
			local propertyName = nextNullstring()

			local propertyValueIsReferent = buffer.readu8(payloadBuffer, loc) == TYPE_ID.Ref
			if propertyValueIsReferent then
				if not latePropertiesMap[instanceReferent] then
					latePropertiesMap[instanceReferent] = {}
				end

				latePropertiesMap[instanceReferent][propertyName] = nextVariant({ TYPE_ID.Ref })
			else
				propertiesMap[propertyName] = nextVariant()
			end

			-- print(propertyName)
			-- print(propertyName, propertiesMap[propertyName])

			propertiesLength -= 1
		end

		local instance: Instance = nil
		if className == "DataModel" then
			instance = Instance.new("Model")
		elseif className == "Script" then
			instance = NewScript(propertiesMap.Source or "", nilParentedInstance)
		elseif className == "LocalScript" then
			instance = NewLocalScript(propertiesMap.Source or "", nilParentedInstance)
		elseif className == "ModuleScript" then
			instance = NewModuleScript(propertiesMap.Source or "", nilParentedInstance)
		else
			instance = Instance.new(className)
		end
		referentTree[instanceReferent] = instance

		instance.Name = name

		if propertiesMap.Attributes then
			for attributeName, value in propertiesMap.Attributes do
				instance:SetAttribute(attributeName, value)
			end

			propertiesMap.Attributes = nil
		end

		if propertiesMap.Tags then
			for _, tag in propertiesMap.Tags do
				instance:AddTag(tag)
			end

			propertiesMap.Tags = nil
		end

		for propertyName, propertyValue in propertiesMap do
			-- TODO: add custom handlers
			xpcall(function()
				instance[propertyName] = propertyValue
			end, function(error)
				-- warn(`failed setting property {propertyName} with value {propertyValue}; got error "{error}"`)
			end)
		end

		if parentReferent ~= nil then
			instance.Parent = referentTree[parentReferent]
		else
			assert(rootReferent == nil, "there are multiple root referents in the hierarchy")
			rootReferent = instanceReferent
		end

		return instanceReferent
	end

	-- decode entire buffer
	while true do
		local decodedReferent = decodeInstance()
		-- print(`decoded referent {decodedReferent}{if rootReferent == decodedReferent then " [root]" else ""}`)

		if buffer.len(payloadBuffer) == loc then
			-- print("finished decoding payloadBuffer")
			break
		end
	end

	assert(rootReferent, "no root referent in hierarchy")

	for referent, propertyMap in latePropertiesMap do
		-- late property handling (referent handling)
		for propertyName, propertyValue in propertyMap do
			xpcall(function()
				referentTree[referent][propertyName] = referentTree[propertyValue]
			end, function(error)
				warn(
					`failed setting late property {propertyName} on referent {referent} with value {propertyValue}; got error "{error}"`
				)
			end)
		end
	end

	return referentTree[rootReferent]
end

return decode
