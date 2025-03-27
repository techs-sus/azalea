use clap::{Parser, Subcommand};
use darklua_core::Resources;
use encoder::encode_dom;
use rbx_dom_weak::WeakDom;
use std::path::{Path, PathBuf};
use std::{fs::File, io::BufReader};

mod encoder;
mod spec;

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

	/// Fully encodes a model file into an embeddable script, with optional formatting and minification available.
	GenerateEmbeddableScript {
		#[clap(flatten)]
		options: GenerateOptions,
	},

	/// Generates the full decoder into a file, with optional formatting and minification available.
	GenerateFullDecoder { output: PathBuf },
}

#[derive(clap::Args)]
struct GenerateOptions {
	/// Input model file(s) (.rbxm, .rbxmx)
	#[arg(short, long = "input", num_args = 1..)]
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
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[clap(flatten)]
	global_options: GlobalOptions,

	#[command(subcommand)]
	command: Command,
}

#[must_use]
pub fn read_dom_from_path<T: AsRef<Path>>(path: T) -> WeakDom {
	let path = std::fs::canonicalize(path.as_ref()).expect("failed canonicalizing path");
	let file = BufReader::new(File::open(&path).expect("Failed opening path"));
	let extension = path
		.extension()
		.expect("Invalid file")
		.to_str()
		.expect("Failed &OsStr to &str conversion");

	match extension {
		"rbxm" => rbx_binary::from_reader(file).unwrap(),
		"rbxmx" => rbx_xml::from_reader_default(file).unwrap(),
		_ => panic!("Invalid file extension"),
	}
}

#[must_use]
pub fn get_stylua_config() -> stylua_lib::Config {
	let mut config = stylua_lib::Config::new();
	config.syntax = stylua_lib::LuaVersion::Luau;
	config.call_parentheses = stylua_lib::CallParenType::Always;
	config.indent_type = stylua_lib::IndentType::Tabs;
	config.indent_width = 2;

	config
}

pub fn minify_with_darklua(input: PathBuf) {
	let options = darklua_core::Options::new(&input)
		.with_output(input)
		.with_generator_override(darklua_core::GeneratorParameters::Dense {
			column_span: usize::MAX - 16,
		})
		.with_configuration(darklua_core::Configuration::default());

	darklua_core::process(&Resources::from_file_system(), options).unwrap();
}

fn write_to_luau_file<T: AsRef<Path>>(output: T, source: String, format: bool, minify: bool) {
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
				.unwrap(),
			)
			.unwrap();
		}
		(false, true) => {
			// minify
			std::fs::write(output.as_ref(), source).unwrap();
			minify_with_darklua(output.as_ref().to_path_buf());
		}
		(true, true) => panic!("formatting and minifying at the same time is not supported"),
		(false, false) => std::fs::write(output, source).unwrap(),
	}
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

fn main() {
	let args = Args::parse_from(wild::args());
	let (format, minify) = (args.global_options.format, args.global_options.minify);

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
			let metadata =
				std::fs::metadata(&options.output).expect("failed reading metadata of output path");

			inputs.reserve_exact(options.inputs.len());
			let is_single_file = options.inputs.len() == 1;

			if is_single_file {
				assert!(
					metadata.is_file(),
					"output path is directory, but only a single input was specified"
				);

				inputs.push((options.inputs.into_iter().next().unwrap(), options.output));
			} else {
				assert!(
					metadata.is_dir(),
					"output path is not a directory, but multiple inputs were passed"
				);

				for input in options.inputs {
					let file = format!(
						"{}.{file_extension}",
						input
							.file_stem()
							.expect("input doesn't have a file name")
							.to_str()
							.expect("input file name is not valid utf-8")
					);

					inputs.push((input, options.output.join(file)));
				}
			}
		}

		// only one output, exit here
		Command::GenerateFullDecoder { output } => {
			write_to_luau_file(output, spec::generate_full_decoder(), format, minify);
			std::process::exit(0);
		}
	};

	let is_single_file = inputs.len() == 1;

	match command_type {
		CommandType::Encode {
			specialized_decoder,
		} => {
			for (input, output) in inputs {
				let weak_dom = read_dom_from_path(&input);
				std::fs::write(output, encode_dom(&weak_dom)).unwrap();

				if let Some(ref output) = specialized_decoder {
					write_to_luau_file(
						if is_single_file {
							output.to_owned()
						} else {
							let file = format!(
								"{}.decoder.luau",
								input
									.file_stem()
									.expect("input doesn't have a file name")
									.to_str()
									.expect("input file name is not valid utf-8")
							);
							output.join(file)
						},
						spec::generate_specialized_decoder_for_dom(&weak_dom),
						format,
						minify,
					);
				}
			}
		}

		CommandType::GenerateFullScript => {
			for (input, output) in inputs {
				let weak_dom = read_dom_from_path(&input);

				write_to_luau_file(
					output,
					spec::generate_full_script(&weak_dom),
					format,
					minify,
				);
			}
		}

		CommandType::GenerateEmbeddableScript => {
			for (input, output) in inputs {
				let weak_dom = read_dom_from_path(&input);
				write_to_luau_file(
					output,
					spec::generate_embeddable_script(&weak_dom),
					format,
					minify,
				);
			}
		}

		CommandType::GenerateFullDecoder => {
			// this was already handled
			unreachable!()
		}
	}
}
