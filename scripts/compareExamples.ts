import { $, Glob } from "bun";
import { ZstdInit, ZstdStream } from "@oneidentity/zstd-js";
import chalk from "chalk";

const fileExtensionRegex = /\.[^.]+$/;

const platformBinary =
	process.platform === "win32"
		? "./target/debug/azalea.exe"
		: "./target/debug/azalea";

await $`cargo build --bin azalea --features="base122 cli"`;

await Promise.all([
	await ZstdInit(),
	$`${platformBinary} encode --input examples/*.rbxm --output examples`,
	$`${platformBinary} generate-embeddable-script --input examples/*.rbxm --output examples`,
]);

function formatBytes(bytes: number, decimals = 2) {
	if (bytes === 0) return "0 bytes";
	const nonNegativeDecimals = decimals < 0 ? 0 : decimals;
	const sizes = [
		"bytes",
		"KiB",
		"MiB",
		"GiB",
		"TiB",
		"PiB",
		"EiB",
		"ZiB",
		"YiB",
	];

	const index = Math.floor(Math.log(bytes) / Math.log(1024));

	return `${parseFloat(
		(bytes / Math.pow(1024, index)).toFixed(nonNegativeDecimals)
	)} ${sizes[index]}`;
}

const glob = new Glob("examples/*.rbxm");

const promises = [];
for await (const file of glob.scan()) {
	// const luauFilePath = file.replace(fileExtensionRegex, ".luau");
	const binFilePath = file.replace(fileExtensionRegex, ".bin");
	const binZstFilePath = binFilePath + ".zst";

	promises.push(
		Bun.write(
			Bun.file(binZstFilePath),
			ZstdStream.compress(await Bun.file(binFilePath).bytes(), 22, true)
		)
	);
}

await Promise.all(promises);

for await (const rbxmFilePath of glob.scan()) {
	const binZstFilePath = rbxmFilePath.replace(fileExtensionRegex, ".bin.zst");

	const binZstFileSize = Bun.file(binZstFilePath).size;
	const rbxmFileSize = Bun.file(rbxmFilePath).size;
	const prettyAzaleaText = chalk.magentaBright("(via Azalea)");
	const prettyRobloxText = chalk.ansi256(214)("(via Roblox)");

	if (binZstFileSize > rbxmFileSize) {
		console.log(
			`${chalk.redBright("loss!")} ${chalk.green(
				rbxmFilePath
			)} ${prettyAzaleaText} is smaller than ${chalk.red()} ${prettyRobloxText} by ${formatBytes(
				binZstFileSize - rbxmFileSize
			)}`
		);
	} else {
		console.log(
			`${chalk.cyan("win!")} ${chalk.green(
				binZstFilePath
			)} ${prettyAzaleaText} is smaller than ${chalk.red(
				rbxmFilePath
			)} ${prettyRobloxText} by ${formatBytes(rbxmFileSize - binZstFileSize)}`
		);
	}
}
