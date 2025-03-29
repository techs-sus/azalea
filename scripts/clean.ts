import { Glob } from "bun";

for await (const file of new Glob("encoding/testRbxms/*.{luau,bin}").scan(
	"."
)) {
	await Bun.file(file).delete();
}

for await (const file of new Glob("examples/*.{luau,bin,zst}").scan(".")) {
	await Bun.file(file).delete();
}
