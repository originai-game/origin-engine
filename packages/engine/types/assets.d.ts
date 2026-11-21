// Files a game's TSX imports for their effect on the build, not a value:
// a component's strings (`import "./ability-slot.en.ogt"`), which the cook
// compiles into the locale tables and the bundle leaves out.
declare module "*.ogt" {}
