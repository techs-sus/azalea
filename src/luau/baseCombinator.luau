local s = string.char(
	20,
	45,
	37,
	127,
	105,
	16,
	22,
	89,
	194,
	128,
	202,
	134,
	43,
	49,
	88,
	111,
	16,
	29,
	109,
	119,
	19,
	49,
	72,
	104,
	52,
	71,
	86,
	32
)

local b = require("./hackZstd")(require("./base122")(s))
print(buffer.len(b))
print(buffer.readstring(b, 0, buffer.len(b)))
