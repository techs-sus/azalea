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
		load: (script: ModuleScript, require: typeof(oldRequire)) -> () -> unknown,
		cache: unknown,
	}

	type Ref = number

	function TrackModuleScript(ref: Ref, moduleScript: ModuleScript)
		MODULE_SCRIPT_TO_REF[moduleScript] = ref
		return moduleScript
	end

	function NewModuleScript(source: nil, parent: Instance?)
		assert(
			type(source) == "nil",
			"first argument (source) should be nil because the Novel technique inlines sources"
		)

		assert(
			typeof(parent) == "Instance" or parent == nil,
			"second optional argument (parent) must be of type Instance if provided"
		)

		local moduleScript = Instance.new("ModuleScript")
		moduleScript.Parent = parent

		return moduleScript
	end

	-- There is some bug in Roblox preventing require aliases
	-- Based off of: https://gist.github.com/techs-sus/8988a086745f4aa7692aa35e8e8d7d68
	local function getPathTranslaterBoundToScript(script: ModuleScript)
		return function(value: string | Instance | number | unknown)
			if type(value) == "string" then
				local split = string.split(value, "/")
				local start

				local firstPiece = table.remove(split, 1)

				if not firstPiece then
					error("Cannot require an empty path")
				end

				if firstPiece == "." then
					start = script.Parent
				elseif firstPiece == ".." then
					start = script.Parent.Parent
				elseif firstPiece == "@self" then
					start = script
				else
					error(`Path must begin with './', '../', or '@': received '{firstPiece}'`)
				end

				for _, piece in split do
					-- if piece == "." then start = start end
					if piece == "." then
					-- would be start = start; aka do nothing, but don't fallback to FindFirstChild
					elseif piece == ".." then
						start = start.Parent
					else
						start = start:FindFirstChild(piece, false)
					end
				end

				value = start
			end

			return require(value)
		end
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
				cache = module.load(value, getPathTranslaterBoundToScript(value))()

				module.cache = cache
			end

			return cache
		end

		return assert(oldRequire, "require is not available")(value :: any)
	end
end
