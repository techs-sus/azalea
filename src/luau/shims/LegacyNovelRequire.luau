do
	local MODULE_UNCACHED_LVALUE = newproxy(true)
	getmetatable(MODULE_UNCACHED_LVALUE).__metatable =
		"This metatable denotes an Uncached ModuleScript in the Novel Technique."

	local REF_TO_MODULE = table.freeze({
		--@generate
	})

	local MODULE_SCRIPT_TO_REF: { [ModuleScript]: Ref } = {} -- can't be frozen

	local oldRequire = require

	type Module = {
		load: (script: ModuleScript) -> () -> unknown,
		cache: unknown,
	}

	type Ref = number

	function TrackModuleScript(ref: Ref, moduleScript: ModuleScript)
		MODULE_SCRIPT_TO_REF[moduleScript] = ref
		return moduleScript
	end

	function NewModuleScript(source: nil, parent: Instance?)
		assert(type(source) == "nil", "first argument (source) should be nil because the Novel technique inlines sources")
		assert(
			typeof(parent) == "Instance" or parent == nil,
			"second optional argument (parent) must be of type Instance if provided"
		)

		local moduleScript = Instance.new("ModuleScript")
		moduleScript.Parent = parent

		return moduleScript
	end

	function require(value): unknown
		if typeof(value) == "Instance" then
			local module: Module = assert(
				REF_TO_MODULE[assert(
					MODULE_SCRIPT_TO_REF[value],
					"failed lookup in ModuleScript -> Ref table, maybe TrackModuleScript wasn't called?"
				)],
				"failed lookup in Ref -> Module table, tracked but no linkage"
			)

			local cache = module.cache

			if cache == MODULE_UNCACHED_LVALUE then
				-- we cannot provide perfect errors with pcall:
				-- -> works perfectly in OpenSB
				-- -> mysterious errors in lua sandbox
				-- -> C stack overflow in lua assembling

				-- local success = true
				-- success, cache = pcall(module.load(value))

				-- if not success then
				-- 	return error(`Requested module experienced an error whilst loading: {cache}`, 2)
				-- end

				-- we silently ignore >1 return values
				cache = module.load(value)()

				module.cache = cache
			end

			return cache
		end

		return assert(oldRequire, "require is not available")(value :: any)
	end
end
