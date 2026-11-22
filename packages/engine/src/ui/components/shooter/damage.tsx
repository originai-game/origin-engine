import { For, Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import "./shooter.en.ogt";

export interface DamageHit {
  id: string;
  /** The damage dealt. */
  amount: number;
  /** normal, headshot, shield (it struck a shield) or kill. */
  kind?: string;
}

export interface ShooterDamageProps {
  /**
   * The player's latest hits, newest first. The newest stands largest beside
   * the reticle and the older ones step down and away (a phone shows the
   * newest alone); the game trims the list (three or so) and a new row pops
   * in. Read only.
   */
  hits?: ReadonlyArray<DamageHit>;
}

/**
 * Damage the player deals, beside the reticle: the newest hit's figure
 * largest with its word (Headshot) over it, the older ones stepping down and
 * away from the aim. Read only. The rows' pop-in and fade-out transitions
 * have no TSX surface and are dropped.
 */
export function ShooterDamage(props: ShooterDamageProps) {
  return (
    <View class={`flex-col items-start max-w-full pointer-events-none text-fg`} role="log" label={t("shooter.dealt.name")}>
      <For each={props.hits ?? []} key={(h) => h.id}>
        {(h, i) => (
          <View
            class={`flex-col items-start ml-0 animate-pop transition duration-200 exit:opacity-0 data-later:ml-9 data-later:opacity-80 data-third:ml-16 data-third:opacity-60 compact:data-later:hidden`}
            data-later={i() >= 1}
            data-third={i() >= 2}
          >
            <Show when={i() < 1 && h().kind == "headshot"}>
              <Text class={`text-xs font-semibold tracking-label uppercase whitespace-nowrap text-gold text-shadow-2xs text-stroke-3 text-stroke-bg compact:tracking-normal`}>
                {t("shooter.dealt.headshot")}
              </Text>
            </Show>
            <Show when={i() < 1 && h().kind == "kill"}>
              <Text class={`text-xs font-semibold tracking-label uppercase whitespace-nowrap text-gold text-shadow-2xs text-stroke-3 text-stroke-bg compact:tracking-normal`}>
                {t("shooter.dealt.kill")}
              </Text>
            </Show>
            <Text
              class={`text-display-lg leading-none font-bold tabular-nums whitespace-nowrap text-fg text-shadow-2xs text-stroke-3 text-stroke-bg data-headshot:text-gold data-kill:text-gold data-shield:text-ally-ink compact:text-2xl`}
              data-headshot={h().kind == "headshot"}
              data-kill={h().kind == "kill"}
              data-shield={h().kind == "shield"}
            >
              {formatNumber(h().amount)}
            </Text>
          </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<ShooterDamageProps>> = {
  default: {"hits":[{"id":"d2","amount":86,"kind":"headshot"},{"id":"d1","amount":31,"kind":"normal"}]},
  shield: {"hits":[{"id":"s1","amount":24,"kind":"shield"}]},
  kill: {"hits":[{"id":"k3","amount":112,"kind":"kill"},{"id":"k2","amount":45,"kind":"normal"},{"id":"k1","amount":45,"kind":"normal"}]},
  overflow: {"hits":[{"id":"o1","amount":1234567,"kind":"headshot"},{"id":"o2","amount":99999,"kind":"shield"}]},
};
