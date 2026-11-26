// The same typed factories and tokens as the game API, without a runtime import.
import type * as Api from "../types/index.d.ts";
import { defineRecord, fieldSpec } from "./records.ts";

export * from "./generated/exports.ts";

export const str = ((capacity, value = "", options) => Object.freeze({ ...fieldSpec("str", value, options), capacity })) as typeof Api.str;
export const list = ((of, capacity, options) =>
  Object.freeze({ kind: of.kind, default: Object.freeze([]), options: Object.freeze({ ...(options ?? {}) }), list: capacity })) as typeof Api.list;

export const defineComponent = ((name, fields) => defineRecord("component", name, fields)) as typeof Api.defineComponent;
export const defineResource = ((name, fields) => defineRecord("resource", name, fields)) as typeof Api.defineResource;
export const defineEvent = ((name, fields) => defineRecord("event", name, fields)) as typeof Api.defineEvent;
