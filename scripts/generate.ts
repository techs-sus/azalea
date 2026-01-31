import { Glob, $ } from "bun";
import { ZstdInit, ZstdStream } from "@oneidentity/zstd-js";

const platformBinary =
	process.platform === "win32"
		? "./target/debug/azalea.exe"
		: "./target/debug/azalea";

await $`cargo build`;
await Promise.all([
	ZstdInit(),
	$`${platformBinary} generate-full-decoder encoding/decoder.luau --format`,
	$`${platformBinary} encode --input encoding/testRbxms/*.rbxm --output encoding/testRbxms`,
]);

const glob = new Glob("encoding/testRbxms/*.rbxm");
const fileExtensionRegex = /\.[^.]+$/;

for await (const file of glob.scan(".")) {
	const binFilePath = file.replace(fileExtensionRegex, ".bin");

	const encodedBytes = Buffer.from(
		ZstdStream.compress(await Bun.file(binFilePath).bytes(), 22, true),
	).toString("base64");

	await Bun.write(
		file.replace(fileExtensionRegex, ".luau"),
		`return game:GetService("HttpService"):JSONDecode([[{"m":null,"t":"buffer","zbase64":"${encodedBytes}"}]]) :: buffer`,
	);
}
