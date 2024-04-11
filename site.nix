{ buildNpmPackage }:
buildNpmPackage rec {
  pname = "vueko-frontend";
  version = "1.0";
  src = ./vueko-frontend;
  npmFlags = [ "--legacy-peer-deps" ];
  npmDepsHash = "sha256-LAe68o8qbe7He85mBpPLXnaMR0fgO3LsA3/oUkNRFgg=";
}
