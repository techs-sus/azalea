{
  lib,
  rustPlatform,
  ...
}:
rustPlatform.buildRustPackage {
  pname = "azalea";
  version = "0.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

	buildFeatures = [
		"cli"
		"base122"
	];

  nativeBuildInputs = [ ];
  buildInputs = [ ];

  meta = {
    description = "Azalea is an experimental software suite designed to manipulate Roblox model files for use within restricted Roblox environments.";
    homepage = "https://github.com/techs-sus/azalea";
    license = lib.licenses.asl20; # apache license 2.0
    maintainers = [
      {
        name = "techs-sus";
        github = "techs-sus";
        githubId = 92276908;
      }
    ];
    platforms = lib.platforms.unix;
    mainProgram = "azalea";
  };
}
