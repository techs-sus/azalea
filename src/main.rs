use azalea::{encoder, spec};
use clap::{Parser, Subcommand};
use color_eyre::eyre::{self, Context, bail, ensure, eyre};
use darklua_core::rules::{
	ConvertLuauNumber, RemoveCompoundAssignment, RemoveContinue, RemoveFloorDivision,
	RemoveIfExpression, RemoveInterpolatedString, RemoveTypes, Rule,
};
use darklua_core::{Configuration, Options, Resources};
use encoder::encode_dom_into_writer;
use rbx_dom_weak::WeakDom;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::{fs::File, io::BufReader};

#[derive(Subcommand)]
enum Command {
	/// Encode a model file into the custom binary format.
	Encode {
		#[clap(flatten)]
		options: GenerateOptions,

		/// Optional output location for a specialized decoder designed for the input model(s)
		#[arg(short, long)]
		specialized_decoder: Option<PathBuf>,
	},

	/// Fully encodes a model file into a singular Roblox script.
	GenerateFullScript {
		#[clap(flatten)]
		options: GenerateOptions,
	},

	/// Fully encodes a model file into an embeddable script, with optional formatting, minification and compat available.
	GenerateEmbeddableScript {
		#[clap(flatten)]
		options: GenerateOptions,
	},

	/// Generates the full decoder into a file, with optional formatting, minification and compat available.
	GenerateFullDecoder { output: PathBuf },
}

#[derive(clap::Args)]
struct GenerateOptions {
	/// Input model file(s) (.rbxm, .rbxmx)
	#[arg(short, long = "input", num_args = 1.., required = true)]
	inputs: Vec<PathBuf>,

	/// Output luau file / directory
	#[arg(short, long)]
	output: PathBuf,
}

#[derive(clap::Args)]
struct GlobalOptions {
	/// Uses stylua_lib to format
	#[arg(
		short,
		long,
		default_value_t = false,
		global = true,
		conflicts_with = "minify"
	)]
	format: bool,

	/// Uses darklua_core to minify
	#[arg(
		short,
		long,
		default_value_t = false,
		global = true,
		conflicts_with = "format"
	)]
	minify: bool,

	/// Uses darklua_core to emit Lua 5.1 code
	#[arg(short, long, default_value_t = false, global = true)]
	compat: bool,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[clap(flatten)]
	global_options: GlobalOptions,

	#[command(subcommand)]
	command: Command,
}

fn read_dom_from_path<T: AsRef<Path>>(path: T) -> eyre::Result<WeakDom> {
	let path = path.as_ref();
	let file = BufReader::new(
		File::open(path).with_context(|| format!("failed opening path {}", path.display()))?,
	);

	let extension = path
		.extension()
		.ok_or_else(|| eyre!("file {} has no extension", path.display()))?
		.to_str()
		.ok_or_else(|| {
			eyre!(
				"failed &OsStr to &str conversion for path {}",
				path.display()
			)
		})?;

	Ok(match extension {
		"rbxm" => rbx_binary::from_reader(file)?,
		"rbxmx" => rbx_xml::from_reader_default(file)?,
		_ => bail!("invalid file extension"),
	})
}

#[must_use]
fn get_stylua_config() -> stylua_lib::Config {
	let mut config = stylua_lib::Config::new();
	config.syntax = stylua_lib::LuaVersion::Luau;
	config.call_parentheses = stylua_lib::CallParenType::Always;
	config.indent_type = stylua_lib::IndentType::Tabs;
	config.indent_width = 2;

	config
}

fn minify_with_darklua(target: PathBuf) -> Result<(), darklua_core::DarkluaError> {
	let options = darklua_core::Options::new(&target)
		.with_output(target)
		.with_generator_override(darklua_core::GeneratorParameters::Dense {
			column_span: usize::MAX - 16,
		})
		.with_configuration(darklua_core::Configuration::default());

	darklua_core::process(&Resources::from_file_system(), options)?;
	Ok(())
}

fn write_to_luau_file<T: AsRef<Path>>(
	output: T,
	source: String,
	format: bool,
	minify: bool,
	lua_5_1_compat: bool,
) -> eyre::Result<()> {
	let source = if lua_5_1_compat {
		let resources = Resources::from_memory();
		let input_path = Path::new("/src");

		resources
			.write("/src/main.lua", &source)
			.map_err(|e| eyre!("failed to write into memory: {e:?}"))?;

		let config = Configuration::empty()
			.with_rule(Box::new(RemoveTypes::default()) as Box<dyn Rule>)
			.with_rule(Box::new(RemoveFloorDivision::default()) as Box<dyn Rule>)
			.with_rule(Box::new(RemoveCompoundAssignment::default()) as Box<dyn Rule>)
			.with_rule(Box::new(RemoveContinue::default()) as Box<dyn Rule>)
			.with_rule(Box::new(RemoveIfExpression::default()) as Box<dyn Rule>)
			.with_rule(Box::new(RemoveInterpolatedString::default()) as Box<dyn Rule>)
			.with_rule(Box::new(ConvertLuauNumber::default()) as Box<dyn Rule>);

		darklua_core::process(
			&resources,
			Options::new(input_path).with_configuration(config),
		)
		.map_err(|e| eyre!("failed to process with darklua: {e}"))?;

		resources
			.get("/src/main.lua")
			.map_err(|e| eyre!("failed to get output: {e:?}"))?
	} else {
		source
	};

	match (format, minify) {
		(true, false) => {
			// format
			std::fs::write(
				output.as_ref(),
				stylua_lib::format_code(
					&source,
					get_stylua_config(),
					None,
					stylua_lib::OutputVerification::None,
				)
				.context("failed formatting luau source")?,
			)
			.context("failed writing formatted luau output")?;
		}
		(false, true) => {
			// minify
			std::fs::write(output.as_ref(), source).wrap_err("failed writing minified luau output")?;
			minify_with_darklua(output.as_ref().to_path_buf()).map_err(|e| eyre!(e.to_string()))?;
		}
		(true, true) => bail!("formatting and minifying at the same time is not supported"),
		(false, false) => std::fs::write(&output, source).context(format!(
			"failed writing luau source file to output path {}",
			output.as_ref().display()
		))?,
	}

	Ok(())
}

enum CommandType {
	Encode {
		specialized_decoder: Option<PathBuf>,
	},
	GenerateFullScript,
	GenerateEmbeddableScript,
	GenerateFullDecoder,
}

impl Command {
	fn command_type(&self) -> CommandType {
		match self {
			Self::Encode {
				specialized_decoder,
				..
			} => CommandType::Encode {
				specialized_decoder: specialized_decoder.to_owned(),
			},
			Self::GenerateFullScript { .. } => CommandType::GenerateFullScript,
			Self::GenerateEmbeddableScript { .. } => CommandType::GenerateEmbeddableScript,
			Self::GenerateFullDecoder { .. } => CommandType::GenerateFullDecoder,
		}
	}
}

fn main() -> eyre::Result<()> {
	color_eyre::install()?;

	let args = Args::parse_from(wild::args());
	let (format, minify, compat) = (
		args.global_options.format,
		args.global_options.minify,
		args.global_options.compat,
	);

	// Vec<(input, output)>
	let mut inputs = vec![];

	let file_extension = match &args.command {
		Command::Encode { .. } => "bin",
		Command::GenerateFullScript { .. } | Command::GenerateEmbeddableScript { .. } => "luau",
		Command::GenerateFullDecoder { .. } => "",
	};

	let command_type = args.command.command_type();

	// ensure single input -> single file, and multiple inputs -> single directory
	match args.command {
		// commands which can take multiple inputs
		Command::Encode { options, .. }
		| Command::GenerateFullScript { options }
		| Command::GenerateEmbeddableScript { options } => {
			let metadata = std::fs::metadata(&options.output);

			inputs.reserve_exact(options.inputs.len());

			let is_single_file = options.inputs.len() == 1;

			if is_single_file {
				if let Ok(metadata) = metadata {
					ensure!(
						metadata.is_file(),
						"output path is directory, but only a single input was specified"
					);
				}

				inputs.push((options.inputs.into_iter().next().unwrap(), options.output));
			} else {
				ensure!(metadata.is_ok(), "output path does not exist");
				ensure!(
					metadata.unwrap().is_dir(),
					"output path is not a directory, but multiple inputs were passed"
				);

				for input in options.inputs {
					let file = format!(
						"{}.{file_extension}",
						input
							.file_stem()
							.ok_or_else(|| eyre!("input {} doesn't have a file name", input.display()))?
							.to_str()
							.ok_or_else(|| eyre!("input {} has a invalid utf-8 file name", input.display()))?
					);

					inputs.push((input, options.output.join(file)));
				}
			}
		}

		// only one output, exit here
		Command::GenerateFullDecoder { output } => {
			write_to_luau_file(
				output,
				spec::generate_full_decoder(),
				format,
				minify,
				compat,
			)?;
			return Ok(());
		}
	};

	let is_single_file = inputs.len() == 1;

	match command_type {
		CommandType::Encode {
			specialized_decoder,
		} => {
			for (input, output) in inputs {
				let weak_dom = read_dom_from_path(&input)?;
				let mut output_writer =
					BufWriter::new(File::create(&output).wrap_err("failed opening output file for writing")?);

				encode_dom_into_writer(&weak_dom, &mut output_writer)
					.with_context(|| format!("failed encoding dom into output path {}", output.display()))?;

				// "It is critical to call flush before BufWriter<W> is dropped." - BufWriter documentation
				output_writer.flush()?;

				if let Some(ref output) = specialized_decoder {
					write_to_luau_file(
						if is_single_file {
							output.to_owned()
						} else {
							let file = format!(
								"{}.decoder.luau",
								input
									.file_stem()
									.ok_or_else(|| eyre!("input {} doesn't have a file name", input.display()))?
									.to_str()
									.ok_or_else(|| eyre!("input {} file name is not valid utf-8", input.display()))?
							);
							output.join(file)
						},
						spec::generate_specialized_decoder_for_dom(&weak_dom),
						format,
						minify,
						compat,
					)?;
				}
			}
		}

		CommandType::GenerateFullScript => {
			for (input, output) in inputs {
				let weak_dom = read_dom_from_path(&input)?;

				write_to_luau_file(
					output,
					spec::generate_full_script(&weak_dom),
					format,
					minify,
					compat,
				)?;
			}
		}

		CommandType::GenerateEmbeddableScript => {
			for (input, output) in inputs {
				let weak_dom = read_dom_from_path(&input)?;

				write_to_luau_file(
					output,
					spec::generate_embeddable_script(&weak_dom),
					format,
					minify,
					compat,
				)?;
			}
		}

		CommandType::GenerateFullDecoder => {
			// this was already handled
			unreachable!()
		}
	}

	Ok(())
}
