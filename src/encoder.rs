use crate::spec::TypeId;
use color_eyre::eyre::{self, OptionExt, WrapErr};
use rbx_dom_weak::{
	types::{BinaryString, ContentType, Ref, Variant},
	Instance, WeakDom,
};
use std::{collections::HashMap, io::Write};

mod write_string {
	macro_rules! generate_write_string {
		($target:ident) => {
			pub fn $target<T: ::std::io::Write>(
				mut target: T,
				string: &str,
			) -> ::color_eyre::eyre::Result<()> {
				let bytes = string.as_bytes();
				let len: $target = TryFrom::try_from(bytes.len())?;
				target.write_all(&len.to_le_bytes())?;
				target.write_all(bytes)?;

				Ok(())
			}
		};
	}

	generate_write_string!(u8);
	generate_write_string!(u16);
	generate_write_string!(u32);
	generate_write_string!(u64);
	generate_write_string!(u128);
}

/// NOTE: This function does not add any sort of type id.
fn write_varstring(target: &mut impl Write, string: &str) -> eyre::Result<()> {
	let len = string.len();
	if len <= u8::MAX.into() {
		target.write_all(&[1])?;
		write_string::u8(target, string)?;
	} else if len <= u16::MAX.into() {
		target.write_all(&[2])?;
		write_string::u16(target, string)?;
	} else if len <= u32::MAX.try_into()? {
		target.write_all(&[4])?;
		write_string::u32(target, string)?;
	} else if len <= u64::MAX.try_into()? {
		target.write_all(&[8])?;
		write_string::u64(target, string)?;
	} else if (len as u128) <= u128::MAX {
		target.write_all(&[16])?;
		write_string::u128(target, string)?;
	} else {
		eyre::bail!(
			"varstring lengths over {} (u128::MAX) are not support",
			u128::MAX
		)
	}

	Ok(())
}

fn write_nullstring(target: &mut impl Write, string: &str) -> eyre::Result<()> {
	target
		.write_all(string.as_bytes())
		.wrap_err("failed writing string contents for nullstring")?;
	target
		.write_all(&[0])
		.wrap_err("failed writing null byte for nullstring")?;

	Ok(())
}

fn write_variant(
	target: &mut impl Write,
	variant: Variant,
	referent_map: &mut HashMap<Ref, usize>,
) -> eyre::Result<()> {
	match variant {
		// Attribute + String name -> Variant
		Variant::Attributes(attributes) => {
			target
				.write_all(&[TypeId::Attributes as u8])
				.wrap_err("failed writing type id for Attributes")?;
			target
				.write_all(
					&u16::try_from(attributes.len())
						.wrap_err("failed truncating attributes length to u16")?
						.to_le_bytes(),
				)
				.wrap_err("failed writing attributes length")?;
			for (attribute_name, attribute_variant) in attributes {
				write_nullstring(target, &attribute_name)
					.wrap_err("failed writing attribute name as nullstring")?;
				write_variant(target, attribute_variant, referent_map).wrap_err_with(|| {
					format!("failed writing attribute variant for attribute name {attribute_name}")
				})?;
			}
		}
		Variant::Axes(axes) => {
			target
				.write_all(&[TypeId::Axes as u8, axes.bits()])
				.wrap_err("failed writing bytes for Axes")?;
		}
		Variant::BinaryString(binary_string) => {
			target
				.write_all(&[TypeId::BinaryString as u8])
				.wrap_err("failed writing type id for BinaryString")?;

			let vector = binary_string.into_vec();
			target
				.write_all(
					&u32::try_from(vector.len())
						.wrap_err("failed truncating BinaryString length to u64")?
						.to_le_bytes(),
				)
				.wrap_err("failed writing byte length for BinaryString")?;
			target
				.write_all(&vector)
				.wrap_err("failed writing bytes for BinaryString")?;
		}
		Variant::Bool(bool) => {
			target
				.write_all(&[TypeId::Bool as u8, bool.into()])
				.wrap_err("failed writing bytes for Bool")?;
		}
		Variant::BrickColor(brick_color) => {
			let name = format!("{brick_color}");
			target
				.write_all(&[TypeId::BrickColor as u8])
				.wrap_err("failed writing type id for BrickColor")?;
			write_nullstring(target, &name)
				.wrap_err("failed writing name for BrickColor as nullstring")?;
		}
		Variant::CFrame(cframe) => {
			target
				.write_all(&[
					TypeId::CFrame as u8,
					cframe
						.orientation
						.to_basic_rotation_id()
						.unwrap_or_default(),
				])
				.wrap_err("failed writing init bytes for CFrame")?;

			match cframe.orientation.to_basic_rotation_id() {
				None => {
					// write id 0
					// https://dom.rojo.space/binary.html
					target
						.write_all(
							[
								cframe.orientation.x.x,
								cframe.orientation.x.y,
								cframe.orientation.x.z,
								cframe.orientation.y.x,
								cframe.orientation.y.y,
								cframe.orientation.y.z,
								cframe.orientation.z.x,
								cframe.orientation.z.y,
								cframe.orientation.z.z,
								cframe.position.x,
								cframe.position.y,
								cframe.position.z,
							]
							.map(f32::to_le_bytes)
							.as_flattened(),
						)
						.wrap_err("failed writing bytes for CFrame (id = 0x00) orientation + position")?;
				}
				Some(id) => {
					target
						.write_all(
							[cframe.position.x, cframe.position.y, cframe.position.z]
								.map(f32::to_le_bytes)
								.as_flattened(),
						)
						.wrap_err_with(|| format!("failed writing bytes for CFrame (id = {id:02x})"))?;
				}
			}
		}
		Variant::Color3(color) => {
			target
				.write_all(&[TypeId::Color3 as u8])
				.wrap_err("failed writing type id for Color3")?;
			target
				.write_all(
					[color.r, color.g, color.b]
						.map(f32::to_le_bytes)
						.as_flattened(),
				)
				.wrap_err("failed writing bytes for Color3")?;
		}
		Variant::Color3uint8(color) => {
			target
				.write_all(&[TypeId::Color3uint8 as u8])
				.wrap_err("failed writing type id for Color3uint8")?;
			target
				.write_all(
					[color.r, color.g, color.b]
						.map(u8::to_le_bytes)
						.as_flattened(),
				)
				.wrap_err("failed writing bytes for Color3uint8")?;
		}
		Variant::ColorSequence(sequence) => {
			target
				.write_all(&[TypeId::ColorSequence as u8])
				.wrap_err("failed writing type id for ColorSequence")?;

			target
				.write_all(
					&u16::try_from(sequence.keypoints.len())
						.wrap_err("failed truncating ColorSequence length to u16")?
						.to_le_bytes(),
				)
				.wrap_err("failed writing ColorSequence length")?;

			for keypoint in sequence.keypoints {
				let color = keypoint.color;

				target
					.write_all(
						[keypoint.time, color.r, color.g, color.b]
							.map(f32::to_le_bytes)
							.as_flattened(),
					)
					.wrap_err("failed writing bytes for ColorSequenceKeypoint within ColorSequence")?;
			}
		}
		Variant::ContentId(content) => {
			// "When exposed to Lua, this is just a string."
			write_variant(target, Variant::String(content.into_string()), referent_map)?;
		}
		Variant::Content(content) => {
			match content.value() {
				ContentType::None => {
					target
						.write_all(&[TypeId::None as u8])
						.wrap_err("failed to write type id for nil Content")?;
				}
				ContentType::Object(referent) => {
					write_variant(target, Variant::Ref(*referent), referent_map)?;
				}
				ContentType::Uri(string) => {
					target
						.write_all(&[TypeId::String as u8])
						.wrap_err("failed to write type id for uri Content")?;
					write_varstring(target, string)
						.wrap_err("failed writing varstring for uri Content string")?;
				}
				_ => todo!("ContentType {:#?} is not yet implemented", content.value()),
			};
		}
		Variant::Enum(enumeration) => {
			// u32 internally
			target
				.write_all(&[TypeId::Enum as u8])
				.wrap_err("failed writing type id for Enum")?;
			target
				.write_all(&enumeration.to_u32().to_le_bytes())
				.wrap_err("failed writing bytes for Enum")?;
		}
		Variant::EnumItem(enumeration) => {
			// u32 internally
			target
				.write_all(&[TypeId::Enum as u8])
				.wrap_err("failed writing type id for EnumItem")?;
			target
				.write_all(&enumeration.value.to_le_bytes())
				.wrap_err("failed writing bytes for EnumItem")?;
		}
		Variant::Faces(faces) => {
			target
				.write_all(&[TypeId::Faces as u8, faces.bits()])
				.wrap_err("failed writing bytes for Faces")?;
		}
		Variant::Float32(float) => {
			target
				.write_all(&[TypeId::Float32 as u8])
				.wrap_err("failed to write type id for Float32")?;
			target
				.write_all(&float.to_le_bytes())
				.wrap_err("failed to write bytes for Float32")?;
		}
		Variant::Float64(float) => {
			target
				.write_all(&[TypeId::Float64 as u8])
				.wrap_err("failed to write type id for Float64")?;
			target
				.write_all(&float.to_le_bytes())
				.wrap_err("failed to write bytes for Float64")?;
		}
		Variant::Font(font) => {
			target
				.write_all(&[TypeId::Font as u8])
				.wrap_err("failed writing type id for Font")?;

			write_nullstring(target, &font.family)
				.wrap_err("failed writing family for Font as nullstring")?;

			target
				.write_all(&font.weight.as_u16().to_le_bytes())
				.wrap_err("failed writing font weight for Font")?;

			target
				.write_all(&[font.style.as_u8()])
				.wrap_err("failed writing font style for Font")?;
		}
		Variant::Int32(int) => {
			target
				.write_all(&[TypeId::Int32 as u8])
				.wrap_err("failed to write type id for Int32")?;
			target
				.write_all(&int.to_le_bytes())
				.wrap_err("failed writing bytes for Int32")?;
		}
		Variant::Int64(int) => write_variant(
			target,
			Variant::Int32(i32::try_from(int).wrap_err("failed truncating int64 to int32")?),
			referent_map,
		)?,
		Variant::MaterialColors(colors) => {
			let bytes = colors.encode();
			let mut target_bytes = Vec::with_capacity(bytes.len() + 1);
			target_bytes.push(TypeId::MaterialColors as u8);
			target_bytes.extend(bytes);

			target
				.write_all(&target_bytes)
				.wrap_err("failed writing bytes for MaterialColors")?;
		}
		Variant::NumberRange(range) => {
			target
				.write_all(&[TypeId::NumberRange as u8])
				.wrap_err("failed writing type id for NumberRange")?;
			target
				.write_all([range.min, range.max].map(f32::to_le_bytes).as_flattened())
				.wrap_err("failed writing bytes for NumberRange")?;
		}
		Variant::NumberSequence(sequence) => {
			target
				.write_all(&[TypeId::NumberSequence as u8])
				.wrap_err("failed writing type id for NumberSequence")?;

			target
				.write_all(
					&u16::try_from(sequence.keypoints.len())
						.wrap_err("failed truncating number sequence length to u16")?
						.to_le_bytes(),
				)
				.wrap_err("failed writing number sequence length")?;

			for keypoint in sequence.keypoints {
				target
					.write_all(
						[keypoint.envelope, keypoint.time, keypoint.value]
							.map(f32::to_le_bytes)
							.as_flattened(),
					)
					.wrap_err("failed writing bytes for NumberSequenceKeypoint for NumberSequence")?;
			}
		}
		Variant::OptionalCFrame(cframe) => match cframe {
			None => {
				target
					.write_all(&[TypeId::None as u8])
					.wrap_err("failed writing type id for OptionalCFrame")?;
			}
			Some(cframe) => write_variant(target, Variant::CFrame(cframe), referent_map)?,
		},
		Variant::PhysicalProperties(properties) => match properties {
			rbx_dom_weak::types::PhysicalProperties::Default => {
				target
					.write_all(&[TypeId::DefaultPhysicalProperties as u8])
					.wrap_err("failed writing type id for default PhysicalProperties")?;
			}
			rbx_dom_weak::types::PhysicalProperties::Custom(custom_physical_properties) => {
				target
					.write_all(&[TypeId::CustomPhysicalProperties as u8])
					.wrap_err("failed writing type id for CustomPhysicalProperties")?;
				target
					.write_all(
						[
							custom_physical_properties.density,
							custom_physical_properties.elasticity,
							custom_physical_properties.elasticity_weight,
							custom_physical_properties.friction,
							custom_physical_properties.friction_weight,
						]
						.map(f32::to_le_bytes)
						.as_flattened(),
					)
					.wrap_err("failed writing bytes for CustomPhysicalProperties")?;
			}
		},
		Variant::Ray(ray) => {
			target
				.write_all(&[TypeId::Ray as u8])
				.wrap_err("failed writing type id for Ray")?;
			target
				.write_all(
					[
						ray.direction.x,
						ray.direction.y,
						ray.direction.z,
						ray.origin.x,
						ray.origin.y,
						ray.origin.z,
					]
					.map(f32::to_le_bytes)
					.as_flattened(),
				)
				.wrap_err("failed writing bytes for Ray")?;
		}
		Variant::Rect(rect) => {
			target
				.write_all(&[TypeId::Rect as u8])
				.wrap_err("failed writing type id for Rect")?;
			target
				.write_all(
					[rect.min.x, rect.min.y, rect.max.x, rect.max.y]
						.map(f32::to_le_bytes)
						.as_flattened(),
				)
				.wrap_err("failed writing bytes for Rect")?;
		}
		Variant::Ref(referent) => {
			if referent.is_none() {
				target
					.write_all(&[TypeId::None as u8])
					.wrap_err("failed writing type id for nil Ref")?;
				return Ok(());
			}

			target
				.write_all(&[TypeId::Ref as u8])
				.wrap_err("failed writing type id for Ref (referent)")?;

			// referents are internally stored as random u128 values
			// it is safe to assume that >[`u64::MAX`] ref's will never be created or used in one model at
			// once, therefore, we can map refs to smaller usize id's. furthermore, roblox cannot handle values
			// above [`u32::MAX`].

			let id = match referent_map.get(&referent) {
				Some(&id) => id,
				None => {
					let id = referent_map.len() + 1; /* ensure an id of 0 is never reached, as we filter all zeros */
					referent_map.insert(referent, id);
					id
				}
			};

			leb128::write::unsigned(target, id.try_into()?)
				.wrap_err("failed writing leb128 unsigned integer for Ref")?;

			// target
			// 	.write_all(&[0])
			// 	.wrap_err("failed writing null byte for Ref")?;
		}
		Variant::Region3(region3) => {
			target
				.write_all(&[TypeId::Region3 as u8])
				.wrap_err("failed to write type id for Region3")?;
			target
				.write_all(
					[
						region3.min.x,
						region3.min.y,
						region3.min.z,
						region3.max.x,
						region3.max.y,
						region3.max.z,
					]
					.map(f32::to_le_bytes)
					.as_flattened(),
				)
				.wrap_err("failed to write bytes for Region3")?;
		}
		Variant::Region3int16(region3) => {
			target
				.write_all(&[TypeId::Region3int16 as u8])
				.wrap_err("failed to write type id for Region3int16")?;

			target
				.write_all(
					[
						region3.min.x,
						region3.min.y,
						region3.min.z,
						region3.max.x,
						region3.max.y,
						region3.max.z,
					]
					.map(i16::to_le_bytes)
					.as_flattened(),
				)
				.wrap_err("failed to write bytes for Region3int16")?;
		}
		Variant::SecurityCapabilities(capabilities) => {
			target
				.write_all(&[TypeId::SecurityCapabilities as u8])
				.wrap_err("failed writing type id for SecurityCapabilities")?;
			target
				.write_all(&capabilities.bits().to_le_bytes())
				.wrap_err("failed writing bytes for SecurityCapabilities")?;
		}
		Variant::SharedString(string) => write_variant(
			target,
			Variant::BinaryString(BinaryString::from(string.data())),
			referent_map,
		)?,

		Variant::String(string) => {
			target
				.write_all(&[TypeId::String as u8])
				.wrap_err("failed to write type id for String")?;
			write_varstring(target, &string).wrap_err("failed writing varstring for String")?;
		}
		Variant::Tags(tags) => {
			target
				.write_all(&[TypeId::Tags as u8])
				.wrap_err("failed to write tag id for Tags")?;
			target
				.write_all(
					&u16::try_from(tags.len())
						.wrap_err("failed truncating tags length to u16")?
						.to_le_bytes(),
				)
				.wrap_err("failed writing tag length")?;

			for tag in tags.iter() {
				write_nullstring(target, tag).wrap_err("failed writing tag as nullstring")?;
			}
		}
		Variant::UDim(udim) => {
			target
				.write_all(&[TypeId::UDim as u8])
				.wrap_err("failed writing type id for UDim")?;

			target
				.write_all([udim.offset.to_le_bytes(), udim.scale.to_le_bytes()].as_flattened())
				.wrap_err("failed writing bytes for UDim")?;
		}
		Variant::UDim2(udim2) => {
			target
				.write_all(&[TypeId::UDim2 as u8])
				.wrap_err("failed writing type id for UDim2")?;

			target
				.write_all(
					[
						udim2.x.offset.to_le_bytes(),
						udim2.y.offset.to_le_bytes(),
						udim2.x.scale.to_le_bytes(),
						udim2.y.scale.to_le_bytes(),
					]
					.as_flattened(),
				)
				.wrap_err("failed writing bytes for UDim2")?;
		}
		Variant::UniqueId(id) => {
			let string = format!("{id}");
			write_variant(target, Variant::String(string), referent_map)?;
		}
		Variant::Vector2(vector) => {
			target
				.write_all(&[TypeId::Vector2 as u8])
				.wrap_err("failed writing type id for Vector2")?;
			target
				.write_all([vector.x, vector.y].map(f32::to_le_bytes).as_flattened())
				.wrap_err("failed writing bytes for Vector2")?;
		}
		Variant::Vector2int16(vector) => {
			target
				.write_all(&[TypeId::Vector2int16 as u8])
				.wrap_err("failed writing type id for Vector2int16")?;
			target
				.write_all([vector.x, vector.y].map(i16::to_le_bytes).as_flattened())
				.wrap_err("failed writing bytes for Vector2int16")?;
		}
		Variant::Vector3(vector) => {
			target
				.write_all(&[TypeId::Vector3 as u8])
				.wrap_err("failed writing type id for Vector3")?;
			target
				.write_all(
					[vector.x, vector.y, vector.z]
						.map(f32::to_le_bytes)
						.as_flattened(),
				)
				.wrap_err("failed writing bytes for Vector3")?;
		}
		Variant::Vector3int16(vector) => {
			target
				.write_all(&[TypeId::Vector3int16 as u8])
				.wrap_err("failed writing type id for Vector3int16")?;
			target
				.write_all(
					[vector.x, vector.y, vector.z]
						.map(i16::to_le_bytes)
						.as_flattened(),
				)
				.wrap_err("failed writing bytes for Vector3int16")?;
		}

		_ => eyre::bail!("unimplemented VariantType: {:#?}", variant.ty()),
	}

	Ok(())
}

pub fn encode_instance(
	instance: &Instance,
	weak_dom: &WeakDom,
	referent_map: &mut HashMap<Ref, usize>,
	buffer: &mut impl Write,
) -> eyre::Result<()> {
	write_variant(buffer, Variant::String(instance.name.clone()), referent_map)?;

	write_nullstring(buffer, instance.class.as_str())
		.wrap_err("failed writing nullstring for instance ClassName")?;
	write_variant(buffer, Variant::Ref(instance.referent()), referent_map)?;
	write_variant(buffer, Variant::Ref(instance.parent()), referent_map)?;

	// Properties
	buffer.write_all(
		&(u16::try_from(instance.properties.len())
			.wrap_err("failed truncating properties length to u16")?)
		.to_le_bytes(),
	)?;

	for (property, value) in &instance.properties {
		write_nullstring(buffer, property).wrap_err("failed writing property name as nullstring")?;
		write_variant(buffer, value.to_owned(), referent_map)
			.wrap_err("failed writing property variant")?;
	}

	// Children
	for child in instance.children() {
		let child_instance = weak_dom
			.get_by_ref(child.to_owned())
			.ok_or_eyre("referent must exist")?;

		encode_instance(child_instance, weak_dom, referent_map, buffer)?;
	}

	Ok(())
}

pub fn encode_dom_into_writer(weak_dom: &WeakDom, mut writer: impl Write) -> eyre::Result<()> {
	encode_instance(
		weak_dom.root(),
		weak_dom,
		&mut HashMap::new(),
		writer.by_ref(),
	)
}
