# azalea

Azalea is an **_EXPERIMENTAL_** software suite designed to manipulate Roblox model files for use within restricted Roblox environments such as [OpenSB](https://github.com/Open-SB/OpenSB).

azalea is **_freely_** available to **_you, the consumer_** under the Apache 2.0 license.

> legal note: all code generated by tools, scripts, or code in this repo are to be considered as files under the Apache 2.0 license

## Running it + command guide

```bash
# pull the repo first obviously
git clone https://github.com/techs-sus/azalea
cd azalea

# Applys to all commands:
# -m [--minify] is optional: you can specify if you want to minify the luau result via darklua
# -f [--format] is optional: you can specify if you want to format the luau result via stylua
# using -m and -f together lead to a panic! you can only use one

# -s is optional: you can specify a location for a specialized decoder to be generated
cargo run -- encode -i input.rbxm -o output.bin -s specializedDecoder.luau -m

# generates a full decoder
cargo run -- generate-full-decoder -o output.luau -f
# generates a full script: input.rbxm must be a MainModule or have a root ModuleScript
cargo run -- generate-full-script -i input.rbxm -o output.luau -m
# generates an embeddable script, useful for embedding assets
cargo run -- generate-embeddable-script -i input.rbxm -o output.luau -m
```

### Development notes

flake provides:

- a formatter usable with `nix flake fmt` (formats the entire flake)
- a devshell usable with `nix develop`
- a package usable in your flake via `azalea.packages.${system}.default`

bun is used for:

- generating required files for tests via `bun run generate`
- running tests via `bun run tests`; you need either [`run-in-roblox`](https://github.com/rojo-rbx/run-in-roblox) (preferred) or [`run-in-cloud`](https://github.com/techs-sus/run-in-cloud)

format details:

- roblox compresses chunks using lz4 and zstd, azalea's format is chunkless
- currently OpenSB and Roblox Studio are **offically** supported

format efficency when azalea is compressed at zstd level 22:

- react-lua-17-rel.bin.zst = 266kb; react-lua-17-rel.rbxm is 553kb; (we won by 287kb)
- attributes_and_tags.bin.zst = 26kb; attributes_and_tags.rbxm = 15kb; (we lost by 11kb, probably because roblox can use lz4)
