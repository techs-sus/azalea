import { $, which } from "bun";

await $`rojo build -o place.rbxl ./place.project.json`.quiet();

// run-in-roblox is preferred as it runs locally
const binary = which("run-in-roblox")
	? "run-in-roblox"
	: which("run-in-cloud")
		? "run-in-cloud run" // run-in-cloud is compatible if you add the "run" argument
		: (() => {
				throw new Error(
					"either run-in-roblox or run-in-cloud must be available to run the test suite",
				);
			})();

// avoid bun treating our substitution as an escaped argument
await $`${{
	raw: binary,
}} --place place.rbxl --script ./scripts/runTests.luau`;
