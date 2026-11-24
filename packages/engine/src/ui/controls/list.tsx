import { For, Scroll, Text, View, createMemo, createSignal, formatNumber, packClass } from "@origingame/engine/ui";

export interface ListProps<T> {
  /** The items, in order: thousands cost what the rows in view cost. */
  each: readonly T[];
  /** Each row's height, CSS px: every row is this tall (a leaderboard's, an inventory's rows). */
  itemSize: number;
  /** Rows made beyond each end of the view, so a fast scroll shows no gap (default 4). */
  overscan?: number | undefined;
  /** A row's key, kept while it stays in view (default: an object's `id`, else the item). */
  key?: ((item: T) => unknown) | undefined;
  /** What screen readers call it. */
  label?: string | undefined;
  /** Classes of the document that uses it, on its scroll view: it fills what its parent gives it. */
  class?: string | undefined;
  /** One row: the item and its index in `each`. */
  children: (item: () => T, index: () => number) => any;
}

interface Placed<T> { item: T; at: number }

/**
 * A long list that makes only the rows near its view (NSH's `List`): a
 * scroll view as tall as every row, whose rows in view, and `overscan`
 * more each way, exist and stand at their place; as it scrolls, rows that
 * leave the view go and those that come in are made. Its scroll view
 * draws its own bar (`Scrollbar.*`), and the keys scroll a focused row
 * into view as anywhere.
 */
export function List<T>(props: ListProps<T>) {
  const [top, view] = [createSignal(0), createSignal(0)];
  const over = () => Math.max(0, props.overscan ?? 4);
  const rows = createMemo<Placed<T>[]>(() => {
    const size = Math.max(1, props.itemSize);
    const count = props.each.length;
    // Before the first report, as many rows as a tall phone shows.
    const span = view() > 0 ? view() : 900;
    const first = Math.max(0, Math.floor(top() / size) - over());
    const last = Math.min(count, Math.ceil((top() + span) / size) + over());
    const out: Placed<T>[] = [];
    for (let at = first; at < last; at++) out.push({ item: props.each[at]!, at });
    return out;
  });
  const key = (p: Placed<T>) => (props.key ? props.key(p.item) : itemKey(p.item, p.at));
  return (
    <Scroll
      role="list"
      label={props.label}
      class={`grow shrink basis-0 min-h-0 ${props.class ?? ""}`}
      onScroll={(e) => {
        top.set(e.y);
        view.set(Number(e.value.split(" ")[1]) || 0);
      }}
    >
      <View class="relative w-full shrink-0" style={{ height: `${props.each.length * props.itemSize}px` }}>
        <For each={rows()} key={key}>
          {(p) => (
            <View role="listitem" class="absolute inset-x-0" style={{ top: `${p().at * props.itemSize}px`, height: `${props.itemSize}px` }}>
              {props.children(() => p().item, () => p().at)}
            </View>
          )}
        </For>
      </View>
    </Scroll>
  );
}

function itemKey(item: unknown, at: number): unknown {
  if (item !== null && typeof item === "object") {
    const id = (item as { id?: unknown }).id;
    if (id !== undefined && id !== null) return id;
  }
  return typeof item === "object" ? at : item;
}

const ROWS = Array.from({ length: 5000 }, (_, i) => ({ id: i, name: `Player ${i + 1}`, score: 1_000_000 - i * 137 }));

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

export const fixtures: Record<string, Partial<ListProps<(typeof ROWS)[number]>>> = {
  thousands: {
    each: ROWS,
    itemSize: 48,
    label: "Ranking",
    children: (r, i) => (
      <View class={`flex-row items-center gap-3 h-full px-3 ${packClass("Row", "frame")}`}>
        <Text class="w-14 text-fg-muted tabular-nums">{`#${i() + 1}`}</Text>
        <Text class="grow truncate font-semibold">{r().name}</Text>
        <Text class="tabular-nums font-bold">{formatNumber(r().score)}</Text>
      </View>
    ),
  },
};
