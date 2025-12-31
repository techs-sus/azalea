//! Azalea's code generation logic

use askama::Template;
use rbx_dom_weak::WeakDom;
use rbx_dom_weak::types::Ref;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use crate::spec::{ALL_TYPE_IDS, TypeId, get_luau_for_type_ids, get_luau_variant_decoder_for_ids};

bitflags::bitflags! {
	/// Implicit requirements can be set by consumers of the crate, but they will usually be automatically generated.
	/// Instead, set explicit requirements, as they mainly control the behavior of the generated code, and are not automatically generated.
	// #[repr(transparent)]
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
	pub struct Requirements: u16 {
		/// Enable this if you want to decode anything which references a CFrame value.
		///
		/// This is an IMPLICIT requirement.
		const CFRAME_LOOKUP_TABLE = 1;
		/// Enable this if you want to properly decode Script instances.
		///
		/// This is an IMPLICIT requirement.
		const NEW_SCRIPT_FUNCTION = 2;
		/// Enable this if you want to properly decode LocalScript instances.
		///
		/// This is an IMPLICIT requirement.
		const NEW_LOCAL_SCRIPT_FUNCTION = 4;
		/// Enable this if you want to properly decode ModuleScript instances.
		///
		/// This is an IMPLICIT requirement.
		const NEW_MODULE_SCRIPT_FUNCTION = 8;

		/// Enable this if you want to properly decode MeshPart instances.
		///
		/// This is an IMPLICIT requirement.
		const MESH_PART_SUPPORT = 16;

		/* if NEW_SCRIPT_FUNCTION or NEW_LOCAL_SCRIPT_FUNCTION or NEW_MODULE_SCRIPT_FUNCTION are enabled, one of the below MUST be enabled */

		/// Enable this if you want to run Azalea generated scripts in the command bar.
		///
		/// This is an EXPLICIT requirement.
		const STUDIO_SUPPORT = 32;
		/// Enable this if you want to run Azalea generated scripts in a compliant OpenSB implementation which supports NewScript, NewLocalScript and NewModuleScript.
		///
		/// This is an EXPLICIT requirement.
		const OPENSB_SUPPORT = 64;
		/// Enable this if you want to run Azalea generated scripts in an environment which supports NewScript, NewLocalScript but NOT NewModuleScript.
		/// We shim require, and you'll need to use the shimmed version for this to be useful at all.
		///
		/// This is an EXPLICIT requirement.
		const LEGACY_SUPPORT = 128;

		/// The Novel Method (avoid loadstring, inline) only applies to [`Self::LEGACY_SUPPORT`].
		///
		/// Inlines ModuleScript sources in a manner similar to [Wax](https://github.com/latte-soft/wax) and [Darklua](https://github.com/seaofvoices/darklua).
		/// Avoids performance regressions by using an explicit require and script upvalue in chunk functions.
		/// Require is therefore still shimmed and all rules from [`Self::LEGACY_SUPPORT`] apply.
		///
		/// This is an EXPLICIT requirement.
		const USE_NOVEL_INLINING = 256;

		/// Controls where a `return decode` is emitted. Be careful setting this.
		///
		/// This is an EXPLICIT requirement.
		const RETURN_DECODE = 512;
	}
}

pub struct Options<'options> {
	pub generation_requirements: Requirements,
	pub known_needed_type_ids: HashSet<TypeId>,
	pub module_script_sources: HashMap<usize, &'options str>,
	pub referent_map: HashMap<Ref, usize>,
}

#[derive(Template)]
#[template(path = "decoder.txt")]
struct DecoderTemplate<'template> {
	type_id_table: &'template str,
	new_script_shim: Option<&'template str>,
	new_local_script_shim: Option<&'template str>,
	new_module_script_shim: Option<&'template str>,
	variant_decoder_table: &'template str,

	requirements: Requirements,
}

fn generate_new_script_glue(requirements: Requirements) -> String {
	let mut exprs = vec![];
	if requirements.contains(Requirements::OPENSB_SUPPORT)
		|| requirements.contains(Requirements::LEGACY_SUPPORT)
	{
		exprs.push("NewScript");
	}

	if requirements.contains(Requirements::STUDIO_SUPPORT) {
		exprs.push(
			r#"(game:GetService("RunService"):IsStudio() and (function(code, parent)
		local script = Instance.new("Script")
		script.Source = code
		script.Parent = parent

		return script
	end))"#,
		);
	}

	format!(
		"local NewScript: (code: string, parent: Instance?) -> Script = {}",
		exprs.join(" or ")
	)
}

fn generate_new_local_script_glue(requirements: Requirements) -> String {
	let mut exprs = Vec::with_capacity(2);
	if requirements.contains(Requirements::OPENSB_SUPPORT)
		|| requirements.contains(Requirements::LEGACY_SUPPORT)
	{
		exprs.push("NewLocalScript");
	}

	if requirements.contains(Requirements::STUDIO_SUPPORT) {
		exprs.push(
			r#"(game:GetService("RunService"):IsStudio() and (function(code, parent)
		local script = Instance.new("LocalScript")
		script.Source = code
		script.Parent = parent

		return script
	end))"#,
		);
	}

	format!(
		"local NewLocalScript: (code: string, parent: Instance?) -> LocalScript = {}",
		exprs.join(" or ")
	)
}

fn generate_new_module_script_glue(options: &Options) -> String {
	let mut exprs: Vec<String> = Vec::with_capacity(3);
	match options
		.generation_requirements
		.contains(Requirements::USE_NOVEL_INLINING)
	{
		true => {
			let mut output = String::new();

			for (ref_id, source) in &options.module_script_sources {
				writeln!(
					output,
					"[{ref_id}] = {{ cache = MODULE_UNCACHED_LVALUE, load = function(script: ModuleScript, require: typeof(require)) return function()\n{source}\nend end }},"
				).expect("failed writing module def");
			}

			/* skip other code generation because novel model isn't compatible anyway */
			return include_str!("luau/shims/LegacyNovelRequire.luau").replace("--@generate", &output);
		}

		false => {
			if options
				.generation_requirements
				.contains(Requirements::OPENSB_SUPPORT)
			{
				exprs.push("NewModuleScript".to_string());
			}

			if options
				.generation_requirements
				.contains(Requirements::STUDIO_SUPPORT)
			{
				exprs.push(
					r#"(game:GetService("RunService"):IsStudio() and (function(code, parent)
		local script = Instance.new("ModuleScript")
		script.Source = code
		script.Parent = parent

		return script
	end))"#
						.to_string(),
				);
			}

			if options
				.generation_requirements
				.contains(Requirements::LEGACY_SUPPORT)
			{
				exprs.push(format!(
					"nil\n{};",
					include_str!("luau/shims/LegacyNormalRequire.luau")
				));
			}
		}
	}

	format!(
		"local NewModuleScript: (code: string, parent: Instance?) -> ModuleScript = {}",
		exprs.join(" or ")
	)
}

/// Given an [`Options`], a specialized decoder will be generated for you.
/// You can create the [`Options`] yourself, or get it from [`crate::encoder::encode_dom_into_writer`].
#[must_use]
pub fn generate_with_options(options: &Options) -> String {
	let requirements = options.generation_requirements;
	let mut type_ids = options
		.known_needed_type_ids
		.iter()
		.copied()
		.collect::<Vec<_>>();

	type_ids.sort_unstable();

	let new_script_shim = requirements
		.contains(Requirements::NEW_SCRIPT_FUNCTION)
		.then(|| generate_new_script_glue(requirements));

	let new_local_script_shim = requirements
		.contains(Requirements::NEW_LOCAL_SCRIPT_FUNCTION)
		.then(|| generate_new_local_script_glue(requirements));

	let new_module_script_shim = requirements
		.contains(Requirements::NEW_MODULE_SCRIPT_FUNCTION)
		.then(|| generate_new_module_script_glue(options));

	let template = DecoderTemplate {
		type_id_table: &get_luau_for_type_ids(type_ids.iter()),
		new_script_shim: new_script_shim.as_deref(),
		new_local_script_shim: new_local_script_shim.as_deref(),
		new_module_script_shim: new_module_script_shim.as_deref(),
		variant_decoder_table: &get_luau_variant_decoder_for_ids(type_ids.iter()),
		requirements,
	};

	template.render().unwrap()
}

/// A full decoder requires ModuleScript's to have a Source property.
///
/// You must FULLY encode models (include the Source property!) for them to work with this decoder.
/// Notably, models generated with [`Requirements::USE_NOVEL_INLINING`] exclude the Source property.
#[must_use]
pub fn generate_full_decoder() -> String {
	generate_with_options(&Options {
		generation_requirements: Requirements::all().difference(Requirements::USE_NOVEL_INLINING),
		known_needed_type_ids: HashSet::from(ALL_TYPE_IDS),
		module_script_sources: HashMap::new(),
		referent_map: HashMap::new(),
	})
}

#[cfg(feature = "base122")]
fn internal_create_script(
	weak_dom: &WeakDom,
	base_requirements: Requirements,
	level: u8,
) -> String {
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

	let options =
		crate::encoder::encode_dom_into_writer(weak_dom, &mut encoded_dom, base_requirements)
			.expect("failed encoding dom");

	let mut zstd_out = Vec::with_capacity(encoded_dom.len() / 2);
	let mut zstd_encoder = zstd::Encoder::new(&mut zstd_out, i32::from(level)).unwrap();
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

	// embed decoder
	// output.push_str("local decode = (function()\n");
	output.push_str(&generate_with_options(&options));
	// output.push_str("\nend)()\n");

	// SAFETY: Base122 (and by extension, Base123) encoded data is valid UTF-8.
	let base122 = crate::base122::base123_encode(&zstd_out);
	output.push_str(
		&include_str!("./luau/minifiedCombinator.luau").replace("%REPLACE_ME%", unsafe {
			str::from_utf8_unchecked(&base122)
		}),
	);

	output
}

#[cfg(feature = "base122")]
#[must_use]
pub fn generate_embeddable_script(
	weak_dom: &WeakDom,
	base_requirements: Requirements,
	level: u8,
) -> String {
	format!(
		"{}\nreturn decode(payloadBuffer):GetChildren()[1]\n",
		internal_create_script(weak_dom, base_requirements, level)
	)
}

#[cfg(feature = "base122")]
#[must_use]
pub fn generate_full_script(
	weak_dom: &WeakDom,
	base_requirements: Requirements,
	level: u8,
) -> String {
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

	format!(
		"{}\nreturn require(decode(payloadBuffer):GetChildren()[1])\n",
		internal_create_script(weak_dom, base_requirements, level)
	)
}
