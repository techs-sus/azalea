local ServerScriptService = game:GetService("ServerScriptService")
local JestGlobals = require(ServerScriptService.DevPackages.JestGlobals)
local Decoder = require(ServerScriptService.Decoder.decoder)

local androidRobotMeshPart = require(ServerScriptService.Decoder.testRbxms.AndroidRobotMeshPart)

local test = JestGlobals.test
local expect = JestGlobals.expect

test("ensure MeshPart is decoded correctly", function(_, done)
	local root = Decoder(androidRobotMeshPart)
	expect(root).toEqual(expect.anything())

	local meshPart = root:FindFirstChildWhichIsA("MeshPart")

	expect(meshPart).toEqual(expect.anything())

	expect({
		ClassName = meshPart.ClassName,
		Position = meshPart.Position,
		Size = meshPart.Size,
		DoubleSided = meshPart.DoubleSided,
		MeshId = meshPart.MeshId,
		TextureID = meshPart.TextureID,
		RenderFidelity = meshPart.RenderFidelity,
		CollisionFidelity = meshPart.CollisionFidelity,
	}).toMatchObject({
		ClassName = "MeshPart",
		Position = Vector3.new(0, 2, 0),
		DoubleSided = false,
		MeshId = "rbxassetid://5330516702",
		TextureID = "rbxassetid://5330516767",
		RenderFidelity = Enum.RenderFidelity.Precise,
		CollisionFidelity = Enum.CollisionFidelity.Default,
	})

	done()
end)
