//! Azalea's type id and variant decoder generator

use rbx_dom_weak::types::Variant;
use std::fmt::Write;

// <https://veykril.github.io/tlborm/decl-macros/building-blocks/counting.html#bit-twiddling>
macro_rules! count_tt {
	() => { 0 };
	($odd:tt $($a:tt $b:tt)*) => { (count_tt!($($a)*) << 1) | 1 };
	($($a:tt $even:tt)*) => { count_tt!($($a)*) << 1 };
}

macro_rules! define_type_id {
	($($name:ident = $value:expr,)+) => {
		#[repr(u8)]
		#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
		pub enum TypeId {
			$($name = ($value as u8)),*
		}

		pub const ALL_TYPE_IDS: [TypeId; count_tt!($($name)*)] = [
			$(TypeId::$name),*
		];

		const fn type_id_to_name(id: &TypeId) -> &'static str {
			match id {
				$(TypeId::$name => stringify!($name)),*
			}
		}

		pub fn get_luau_for_type_ids<'a>(ids: impl Iterator<Item = &'a TypeId>) -> String {
			let mut output = String::from("local TYPE_ID = table.freeze({\n");

			for id in ids {
				write!(&mut output, "{} = {},", type_id_to_name(id), *id as u8).unwrap();
			}

			output.push_str("\n})");
			output
		}

	};
}

macro_rules! decode_type_id {
	($($tid:pat => $lua_body:expr,)+) => {

		const fn get_luau_decode_variant_code(id: &TypeId) -> &'static str {
			match id {
				$($tid => $lua_body,)*
			}
		}

		pub fn get_luau_variant_decoder_for_ids<'a>(ids: impl Iterator<Item = &'a TypeId>) -> String {
			let mut output = String::from("-- @generated\nVARIANT_DECODER = table.freeze({\n");

			for id in ids {
				write!(&mut output, "[TYPE_ID.{}] = function()\n{}\nend,\n", type_id_to_name(id), get_luau_decode_variant_code(id)).unwrap();
			}

			output.push_str("\n})");
			output
		}
	};
}

define_type_id! {
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

	ContentNone = 35,
	ContentObject = 36,
	ContentUri = 37,
}

#[must_use]
pub fn variant_to_type_id(variant: &Variant) -> Vec<TypeId> {
	match variant {
		Variant::Axes(..) => vec![TypeId::Axes],
		Variant::BinaryString(..) | Variant::SharedString(..) => vec![TypeId::BinaryString],
		Variant::Bool(..) => vec![TypeId::Bool],
		Variant::BrickColor(..) => vec![TypeId::BrickColor],
		Variant::CFrame(..) => vec![TypeId::CFrame],
		Variant::Color3(..) => vec![TypeId::Color3],
		Variant::Color3uint8(..) => vec![TypeId::Color3uint8],
		Variant::ColorSequence(..) => vec![TypeId::ColorSequence],
		Variant::Enum(..) | Variant::EnumItem(..) => vec![TypeId::Enum],
		Variant::Faces(..) => vec![TypeId::Faces],
		Variant::Float32(..) => vec![TypeId::Float32],
		Variant::Float64(..) => vec![TypeId::Float64],
		Variant::Int32(..) | Variant::Int64(..) => vec![TypeId::Int32],
		Variant::NumberRange(..) => vec![TypeId::NumberRange],
		Variant::NumberSequence(..) => vec![TypeId::NumberSequence],
		Variant::Ray(..) => vec![TypeId::Ray],
		Variant::Rect(..) => vec![TypeId::Rect],
		Variant::Ref(..) => vec![TypeId::Ref],
		Variant::Region3(..) => vec![TypeId::Region3],
		Variant::Region3int16(..) => vec![TypeId::Region3int16],
		Variant::String(..) | Variant::ContentId(..) | Variant::UniqueId(..) => vec![TypeId::String],
		Variant::UDim(..) => vec![TypeId::UDim],
		Variant::UDim2(..) => vec![TypeId::UDim2],
		Variant::Vector2(..) => vec![TypeId::Vector2],
		Variant::Vector2int16(..) => vec![TypeId::Vector2int16],
		Variant::Vector3(..) => vec![TypeId::Vector3],
		Variant::Vector3int16(..) => vec![TypeId::Vector3int16],
		Variant::Font(..) => vec![TypeId::Font],
		Variant::MaterialColors(..) => vec![TypeId::MaterialColors],
		Variant::SecurityCapabilities(..) => vec![TypeId::SecurityCapabilities],
		Variant::Tags(..) => vec![TypeId::Tags],

		Variant::OptionalCFrame(cframe) => vec![match cframe {
			Some(..) => TypeId::CFrame,
			None => TypeId::None,
		}],
		Variant::Content(content) => vec![match content.value() {
			rbx_dom_weak::types::ContentType::None => TypeId::ContentNone,
			rbx_dom_weak::types::ContentType::Object(_) => TypeId::ContentObject,
			rbx_dom_weak::types::ContentType::Uri(_) => TypeId::ContentUri,
			ty => todo!("ContentType {ty:#?} is not yet covered"),
		}],
		Variant::PhysicalProperties(prop) => vec![match prop {
			rbx_dom_weak::types::PhysicalProperties::Default => TypeId::DefaultPhysicalProperties,
			rbx_dom_weak::types::PhysicalProperties::Custom(..) => TypeId::CustomPhysicalProperties,
		}],
		Variant::Attributes(attributes) => {
			let mut tys = attributes
				.iter()
				.flat_map(|(_, variant)| variant_to_type_id(variant))
				.collect::<Vec<TypeId>>();

			tys.push(TypeId::Attributes);

			tys
		}
		_ => todo!("Variant {variant:#?} is not yet covered"),
	}
}

decode_type_id! {
	TypeId::String => r#"
		local stringLength = nextUnsignedInteger()
		loc += stringLength
		return buffer.readstring(payloadBuffer, loc - stringLength, stringLength)
	"#,
	TypeId::BinaryString => r#"
		local stringLength = nextUnsignedInteger()
		loc += stringLength
		return buffer.readstring(payloadBuffer, loc - stringLength, stringLength)
	"#,
	TypeId::None => r#"
		return nil
	"#,
	TypeId::Ref => r#"
		return nextUnsignedInteger()
	"#,
	TypeId::Enum => r#"
		return nextUnsignedInteger()
	"#,
	TypeId::Float32 => r#"
		local float = buffer.readf32(payloadBuffer, loc)
		loc += 4
		return float
	"#,
	TypeId::Float64 => r#"
		local float = buffer.readf64(payloadBuffer, loc)
		loc += 8
		return float
	"#,
	TypeId::Int32 => r#"
		local int = buffer.readi32(payloadBuffer, loc)
		loc += 4
		return int
	"#,
	TypeId::Bool => r#"
		local bool = buffer.readu8(payloadBuffer, loc)
		loc += 1
		return bool == 1
	"#,
	TypeId::Tags => r#"
		-- length of encoded array
		local tagsLength = nextUnsignedInteger()
		local tags = {}

		while tagsLength > 0 do
			local tag = nextNullstring()
			table.insert(tags, tag)

			tagsLength -= 1
		end

		return tags
	"#,
	TypeId::Attributes => r#"
		local attributesLength = nextUnsignedInteger()
		local attributeMap: { [string]: any } = {}

		while attributesLength > 0 do
			local attributeName = nextNullstring()
			attributeMap[attributeName] = nextVariant()
			-- print(attributeName, #attributeName, attributeMap[attributeName])

			attributesLength -= 1
		end

		return attributeMap
	"#,
	TypeId::SecurityCapabilities => r#"
		-- SKIP: SecurityCapabilities is not writable in scripts
		loc += 8
	"#,
	TypeId::BrickColor => r#"
		return BrickColor.new(nextNullstring() :: any)
	"#,
	TypeId::UDim => r#"
		local offset, scale = buffer.readi32(payloadBuffer, loc), buffer.readf32(payloadBuffer, loc + 4)
		loc += 8

		return UDim.new(scale, offset)
	"#,
	TypeId::UDim2 => r#"
		local xOffset, yOffset, xScale, yScale =
			buffer.readi32(payloadBuffer, loc),
			buffer.readi32(payloadBuffer, loc + 4),
			buffer.readf32(payloadBuffer, loc + 8),
			buffer.readf32(payloadBuffer, loc + 12)

		loc += 16

		return UDim2.new(xScale, xOffset, yScale, yOffset)
	"#,
	TypeId::Vector2 => r#"
		local x, y = buffer.readf32(payloadBuffer, loc), buffer.readf32(payloadBuffer, loc + 4)
		loc += 8

		return Vector2.new(x, y)
	"#,
	TypeId::Vector2int16 => r#"
		local x, y = buffer.readi16(payloadBuffer, loc), buffer.readi16(payloadBuffer, loc + 2)
		loc += 4

		return Vector2int16.new(x, y)
	"#,
	TypeId::Vector3 => r#"
		local x, y, z =
			buffer.readf32(payloadBuffer, loc),
			buffer.readf32(payloadBuffer, loc + 4),
			buffer.readf32(payloadBuffer, loc + 8)
		loc += 12

		return Vector3.new(x, y, z)
	"#,
	TypeId::Vector3int16 => r#"
		local x, y, z =
			buffer.readi16(payloadBuffer, loc),
			buffer.readi16(payloadBuffer, loc + 2),
			buffer.readi16(payloadBuffer, loc + 4)
		loc += 6

		return Vector3int16.new(x, y, z)
	"#,
	TypeId::CFrame => r#"
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
	"#,
	TypeId::Color3 => r#"
		local r, g, b =
			buffer.readf32(payloadBuffer, loc),
			buffer.readf32(payloadBuffer, loc + 4),
			buffer.readf32(payloadBuffer, loc + 8)

		loc += 12
		return Color3.new(r, g, b)
	"#,
	TypeId::Color3uint8 => r#"
		local r, g, b =
			buffer.readu8(payloadBuffer, loc),
			buffer.readu8(payloadBuffer, loc + 1),
			buffer.readu8(payloadBuffer, loc + 2)
		loc += 3

		return Color3.fromRGB(r, g, b)
	"#,
	TypeId::Font => r#"
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
	"#,
	TypeId::NumberRange => r#"
		local min, max = buffer.readf32(payloadBuffer, loc), buffer.readf32(payloadBuffer, loc + 4)
		loc += 8

		return NumberRange.new(min, max)
	"#,
	TypeId::Rect => r#"
		local minX, minY, maxX, maxY =
			buffer.readf32(payloadBuffer, loc),
			buffer.readf32(payloadBuffer, loc + 4),
			buffer.readf32(payloadBuffer, loc + 8),
			buffer.readf32(payloadBuffer, loc + 12)

		loc += 16

		return Rect.new(minX, minY, maxX, maxY)
	"#,
	TypeId::Axes => r#"
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
	"#,
	TypeId::Faces => r#"
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
	"#,
	TypeId::Region3 => r#"
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
	"#,
	TypeId::Region3int16 => r#"
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
	"#,
	TypeId::NumberSequence => r#"
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
	"#,
	TypeId::Ray => r#"
		local directionX, directionY, directionZ, originX, originY, originZ =
			buffer.readf32(payloadBuffer, loc),
			buffer.readf32(payloadBuffer, loc + 4),
			buffer.readf32(payloadBuffer, loc + 8),
			buffer.readf32(payloadBuffer, loc + 12),
			buffer.readf32(payloadBuffer, loc + 16),
			buffer.readf32(payloadBuffer, loc + 20)

		loc += 24

		return Ray.new(Vector3.new(originX, originY, originZ), Vector3.new(directionX, directionY, directionZ))
	"#,
	TypeId::MaterialColors => r#"
		-- SKIP: Terrain.MaterialColors is not writable by scripts + we could use :SetMaterialColor(), but theres no reason to support that
		loc += 69
	"#,
	TypeId::ColorSequence => r#"
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
	"#,
	TypeId::DefaultPhysicalProperties => r#"
		-- SKIP: DefaultPhysicalProperties does not translate to Luau without a material
		return nil
	"#,
	TypeId::CustomPhysicalProperties => r#"
		local density, elasticity, elasticityWeight, friction, frictionWeight =
			buffer.readf32(payloadBuffer, loc),
			buffer.readf32(payloadBuffer, loc + 4),
			buffer.readf32(payloadBuffer, loc + 8),
			buffer.readf32(payloadBuffer, loc + 12),
			buffer.readf32(payloadBuffer, loc + 16)

		loc += 20
		return PhysicalProperties.new(density, friction, elasticity, frictionWeight, elasticityWeight)
	"#,
	TypeId::ContentNone => "return Content.none",
	TypeId::ContentObject => "return nextUnsignedInteger()",
	TypeId::ContentUri => "return Content.fromUri(nextNullstring())",
}
