// Corpus 17-missing-ts-import: allowImportingTsExtensions, a file that is not there
import { x } from "./not-here.ts";
export const y = x;
