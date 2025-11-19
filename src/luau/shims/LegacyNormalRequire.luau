(function()
	local oldRequire = require

	type UncachedModule = {
		fn: (...unknown) -> ...unknown,
		args: { unknown },
		ret: nil,
	}

	type CachedModule = {
		fn: nil,
		args: nil,
		ret: { unknown },
	}

	type Module = UncachedModule | CachedModule

	local REGISTERED_MODULES: { [ModuleScript]: Module } = {}

	local function spoofModuleScriptEnvironment(env, moduleScript: ModuleScript)
		return setmetatable({
			script = moduleScript,

			getfenv = function(...)
				return spoofModuleScriptEnvironment(getfenv(...), moduleScript)
			end,
		}, {
			__index = env,
		})
	end

	local function NewModuleScript(source: string, parent: Instance?, ...)
		assert(type(source) == "string", "first argument (source) must be of type string")
		assert(typeof(parent) == "Instance" or parent == nil, "second optional argument (parent) must be of type Instance?")

		local args: { unknown } = { ... }
		local fn = assert(loadstring(source), "failed compiling ModuleScript source")

		local moduleScript = Instance.new("ModuleScript")

		setfenv(fn, spoofModuleScriptEnvironment(getfenv(fn), moduleScript))

		moduleScript.Parent = parent

		REGISTERED_MODULES[moduleScript] = {
			fn = fn,
			args = args,
			ret = nil,
		} :: UncachedModule

		return moduleScript
	end

	function require(value): ...unknown
		if typeof(value) == "Instance" then
			local module = assert(REGISTERED_MODULES[value], "given ModuleScript was not created with NewModuleScript")
			local ret = module.ret

			if not ret then
				local fn = assert(module.fn, "unreachable internal error")
				local args = assert(module.args, "unreachable internal error")

				ret = { pcall(fn, args) }

				if not table.remove(ret, 1) then
					error(`Requested module experienced an error whilst loading: {ret[1]}`)
				end

				REGISTERED_MODULES[value] = {
					fn = nil,
					args = nil,
					ret = ret,
				} :: CachedModule
			end

			assert(ret) --[[ not needed, shuts up typechecker. cant do this assert inline because it breaks unpack??? ]]

			return unpack(ret)
		end

		return assert(oldRequire, "require is not available")(value :: any)
	end

	return NewModuleScript
end)()
