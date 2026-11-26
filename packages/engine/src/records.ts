// Pure record declarations shared by the runtime and the closed cook realm.
// Importing a schema never imports the runtime, opens a host or creates a world.
export interface FieldSpec {
  readonly kind: string;
  readonly default: unknown;
  readonly options: object;
  readonly capacity?: number;
  readonly list?: number;
}

export interface Token {
  readonly kind: "component" | "resource" | "event";
  readonly name: string;
  readonly fields?: Readonly<Record<string, FieldSpec>>;
}

export const fieldSpec = (kind: string, value: unknown, options: unknown): FieldSpec =>
  Object.freeze({ kind, default: value, options: Object.freeze({ ...((options as object | undefined) ?? {}) }) });

const pending: Token[] = [];
let consumer: ((token: Token) => void) | undefined;

/** The runtime subscribes once; declarations made before it loads are replayed. */
export function consumeRecords(receive: (token: Token) => void): void {
  if (consumer) throw new Error("record declarations already have a runtime consumer");
  consumer = receive;
  for (const token of pending.splice(0)) receive(token);
}

export function defineRecord(kind: Token["kind"], name: string, fields: Record<string, FieldSpec>): Token {
  const token: Token = Object.freeze({ kind, name, fields: Object.freeze({ ...fields }) });
  if (consumer) consumer(token);
  else pending.push(token);
  return token;
}
