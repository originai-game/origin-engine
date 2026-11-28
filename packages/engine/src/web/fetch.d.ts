/** Shared web-host transport, also imported unbundled by the lanes pool. */
export declare const FETCH_CONCURRENCY: number;
export declare const FETCH_DEADLINE_MS: number;
/** Share the transport budget with helper and lanes Workers on isolated pages. */
export declare function fetchState(state?: SharedArrayBuffer): SharedArrayBuffer | undefined;
/** Holds a concurrency slot through body consumption; retries transient refusals and network/read errors. */
export declare function fetchResource(url: string | URL | Request, init?: RequestInit, progress?: (loaded: number, total: number) => void): Promise<Response>;
/** The same policy for the blocking split-wasm placeholder, on Workers only. */
export declare function fetchResourceSync(url: string | URL): ArrayBuffer;
