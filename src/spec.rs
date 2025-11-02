//! Azalea's type id and decoder generator

use rbx_dom_weak::{WeakDom, types::Variant};
use std::fmt::Write;
use std::{collections::HashSet, str};

// <https://veykril.github.io/tlborm/decl-macros/building-blocks/counting.html#bit-twiddling>
macro_rules! count_tt {
	() => { 0 };
	($odd:tt $($a:tt $b:tt)*) => { (count_tt!($($a)*) << 1) | 1 };
	($($a:tt $even:tt)*) => { count_tt!($($a)*) << 1 };
}

macro_rules! define_type_id {
	($($name:ident = $value:expr,)+) => {
		#[repr(u8)]
		#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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

		fn get_luau_for_type_ids<'a>(ids: impl Iterator<Item = &'a TypeId>) -> String {
			let mut output = String::from("-- @generated\nlocal TYPE_ID = table.freeze({\n");

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

		fn get_luau_variant_decoder_for_ids<'a>(ids: impl Iterator<Item = &'a TypeId>) -> String {
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
}

fn variant_to_type_id(variant: &Variant) -> Vec<TypeId> {
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
			rbx_dom_weak::types::ContentType::None => TypeId::None,
			rbx_dom_weak::types::ContentType::Uri(_) => TypeId::String,
			rbx_dom_weak::types::ContentType::Object(_) => TypeId::Ref,
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
}

const TEMPLATE_LUAU: &str = include_str!("./luau/decoderTemplate.luau");

// thank you rojo developers: https://dom.rojo.space/binary.html#cframe (god bless)
const CFRAME_LOOKUP_TABLE: &str = r"local CFRAME_ID_LOOKUP_TABLE = table.freeze({
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
})";

const NEW_SCRIPT_SOURCE: &str = r#"local NewScript: (code: string, parent: Instance?) -> Script = NewScript
	or function(code, parent)
		local script = Instance.new("Script")
		script.Source = code
		script.Parent = parent

		return script
	end"#;

const NEW_LOCAL_SCRIPT_SOURCE: &str = r#"local NewLocalScript: (code: string, parent: Instance?) -> LocalScript = NewLocalScript
	or function(code, parent)
		local script = Instance.new("LocalScript")
		script.Source = code
		script.Parent = parent

		return script
	end"#;

const NEW_MODULE_SCRIPT_SOURCE: &str = r#"local NewModuleScript: (code: string, parent: Instance?) -> ModuleScript = NewModuleScript
	or function(code, parent)
		local script = Instance.new("ModuleScript")
		script.Source = code
		script.Parent = parent

		return script
	end"#;

bitflags::bitflags! {
	pub struct Requirements: u8 {
		const CFRAME_LOOKUP_TABLE = 0;
		const NEW_SCRIPT_FUNCTION = 1;
		const NEW_LOCAL_SCRIPT_FUNCTION = 2;
		const NEW_MODULE_SCRIPT_FUNCTION = 3;
	}
}

pub fn generate_with_options<'id>(
	type_ids: impl Iterator<Item = &'id TypeId> + Clone,
	requirements: &Requirements,
) -> String {
	let mut generated_elseif_clauses = String::new();
	if requirements.contains(Requirements::NEW_SCRIPT_FUNCTION) {
		generated_elseif_clauses.push_str("elseif className == \"Script\" then\ninstance = NewScript(propertiesMap.Source or \"\", nilParentedInstance)\n");
	}

	if requirements.contains(Requirements::NEW_LOCAL_SCRIPT_FUNCTION) {
		generated_elseif_clauses.push_str("elseif className == \"LocalScript\" then\ninstance = NewLocalScript(propertiesMap.Source or \"\", nilParentedInstance)\n");
	}

	if requirements.contains(Requirements::NEW_MODULE_SCRIPT_FUNCTION) {
		generated_elseif_clauses.push_str("elseif className == \"ModuleScript\" then\ninstance = NewModuleScript(propertiesMap.Source or \"\", nilParentedInstance)\n");
	}

	TEMPLATE_LUAU
		.replace("--@generate TypeId", &get_luau_for_type_ids(type_ids.clone()))
		.replace(
			"--@generate CFrameIdLookupTable",
			if requirements.contains(Requirements::CFRAME_LOOKUP_TABLE) { CFRAME_LOOKUP_TABLE } else { "-- CFrame lookup table not required" },
		)
		.replace(
			"--@generate NewScript",
			if requirements.contains(Requirements::NEW_SCRIPT_FUNCTION) { NEW_SCRIPT_SOURCE } else { "-- NewScript not required" },
		)
		.replace(
			"--@generate NewLocalScript",
			if requirements.contains(Requirements::NEW_LOCAL_SCRIPT_FUNCTION) {
				NEW_LOCAL_SCRIPT_SOURCE}
				else {"-- NewLocalScript not required"}
		)
		.replace(
			"--@generate NewModuleScript",
			if requirements.contains(Requirements::NEW_MODULE_SCRIPT_FUNCTION) {
				NEW_MODULE_SCRIPT_SOURCE
			} else {"-- NewModuleScript not required"}
		)
		.replace("--@generate NilParentedInstance", if requirements.contains(Requirements::NEW_SCRIPT_FUNCTION | Requirements::NEW_LOCAL_SCRIPT_FUNCTION | Requirements::NEW_MODULE_SCRIPT_FUNCTION) { "local nilParentedInstance = Instance.new(\"Folder\", nil)" } else { "" })
		.replace(
			"--@generate VariantDecoder",
			&get_luau_variant_decoder_for_ids(type_ids),
		).replace("--@generate SpecializedInstanceCreator", &format!("if className == \"DataModel\" then\ninstance = Instance.new(\"Model\")\n{generated_elseif_clauses}\nelse\ninstance = Instance.new(className)\nend"))
}

pub fn generate_full_decoder() -> String {
	generate_with_options(ALL_TYPE_IDS.iter(), &Requirements::all())
}

pub fn generate_specialized_decoder_for_dom(weak_dom: &WeakDom) -> String {
	// we use a hashset to avoid duplicate TypeId's
	let mut type_ids: HashSet<TypeId> = HashSet::from([TypeId::None, TypeId::String, TypeId::Ref]);

	let mut requirements = Requirements::empty();

	for descendant in weak_dom.descendants() {
		match descendant.class.as_str() {
			"Script" => requirements |= Requirements::NEW_SCRIPT_FUNCTION,
			"LocalScript" => requirements |= Requirements::NEW_LOCAL_SCRIPT_FUNCTION,
			"ModuleScript" => requirements |= Requirements::NEW_MODULE_SCRIPT_FUNCTION,

			_ => {}
		}

		for variant in descendant.properties.values() {
			type_ids.extend(variant_to_type_id(variant));
		}
	}

	if type_ids.contains(&TypeId::CFrame) {
		requirements |= Requirements::CFRAME_LOOKUP_TABLE;
	}

	generate_with_options(type_ids.iter(), &requirements)
}

#[cfg(feature = "base122")]
fn internal_create_script(weak_dom: &WeakDom) -> String {
	/*
		* in a perfect world, we would be able to directly wrap writers around each other as below:
		* [[azalea encoder] -> [zstd writer] -> [base64/base122 writer]]
		* all steps above would be perfectly piped
		*
		* however, roblox expects the zstd output to have a pledged src size, however
		* it is impossible to figure out the actual src content size without wasting resources (ram, cpu)
		*
		* currently, the chain would look something like this:
		* [azalea encoder] -> [Vec<u8> (heap)] -> [[zstd writer] -> [base64/base122 writer]]
		* however, the base122 writer is pretty unstable and results in corrupted blocks / checksum failures
		*/

	let mut output = String::new();
	let mut encoded_dom = Vec::new();

	crate::encoder::encode_dom_into_writer(weak_dom, &mut encoded_dom).expect("failed encoding dom");

	let mut zstd_out = Vec::with_capacity(encoded_dom.len() / 2);
	let mut zstd_encoder = zstd::Encoder::new(&mut zstd_out, 22).unwrap();
	zstd_encoder.include_checksum(true).unwrap();
	zstd_encoder.include_contentsize(true).unwrap();
	zstd_encoder
		.set_pledged_src_size(Some(encoded_dom.len() as u64))
		.unwrap();

	std::io::copy(&mut std::io::Cursor::new(encoded_dom), &mut zstd_encoder).unwrap();
	zstd_encoder.finish().unwrap();

	// old base64 generator
	// output.push_str("local payloadBuffer: buffer = game:GetService(\"HttpService\"):JSONDecode([[{\"m\":null,\"t\":\"buffer\",\"zbase64\":\"");
	// output.push_str(&BASE64_STANDARD.encode(&zstd_out));
	// output.push_str("\"}]])\n");

	// SAFETY: Base123 (and by extension, Base122) encoded data is valid UTF-8.
	let base122 = crate::base122::base123_encode(&zstd_out);
	output.push_str(
		&include_str!("./luau/minifiedCombinator.luau").replace("%REPLACE_ME%", unsafe {
			str::from_utf8_unchecked(&base122)
		}),
	);

	// embed decoder
	output.push_str("local decode = (function()\n");
	output.push_str(&generate_specialized_decoder_for_dom(weak_dom));
	output.push_str("\nend)()\n");

	output
}

#[cfg(feature = "base122")]
pub fn generate_embeddable_script(weak_dom: &WeakDom) -> String {
	let mut output = internal_create_script(weak_dom);
	output.push_str("return decode(payloadBuffer):GetChildren()[1]\n");

	output
}

#[cfg(feature = "base122")]
pub fn generate_full_script(weak_dom: &WeakDom) -> String {
	// ensure that the generated script will be requiring a ModuleScript
	{
		let children = weak_dom.root().children();
		assert!(children.len() == 1, "root must have one child");

		let root_first_child = weak_dom.get_by_ref(children[0]).unwrap();

		assert!(
			root_first_child.class == "ModuleScript",
			"DataModel's first child should be a module script"
		);
	};

	let mut output = internal_create_script(weak_dom);

	// require the root ModuleScript
	output.push_str("return require(decode(payloadBuffer):GetChildren()[1])\n");

	output
}
