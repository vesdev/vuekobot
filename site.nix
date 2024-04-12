{ buildNpmPackage, nodejs, bash }:
buildNpmPackage rec {
  pname = "vueko-frontend";
  version = "1.0";
  src = ./vueko-frontend;
  npmInstallFlags = [ "--only-production" ];

  npmDepsHash = "sha256-w1X+a4m0PdsIEIoUmF/xDOh5dPE85GbBQyYCYPXtD8k=";
  npmBuildScript = "build";

  installPhase = ''
    mkdir -p $out/var
    mv build/ $out/var/www
    cp -r package* package* node_modules $out/var/www

    mkdir -p $out/bin
    echo "
      #!${bash}/bin/bash
      NODE_ENV=production ${nodejs}/bin/node $out/var/www/index.js
    " > $out/bin/${pname}
    chmod +xr $out/bin/${pname}
  '';
}
