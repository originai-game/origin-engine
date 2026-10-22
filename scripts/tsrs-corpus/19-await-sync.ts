// Corpus 19-await-sync: await outside an async function
export function f(p: Promise<number>): number { return await p; }
