<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`abilities`](../abilities.md) · [index](../index.md)

<a id="record-Ability"></a>

### `og.Ability` (component)

Something its owner uses: on a press of `action` (og.input.consume, taken only when it can be used, so an early press waits in the buffer) or by og.abilities.use, it spends a charge and its cost and starts its lock, and og.AbilityUsed is sent. Below maxCharges a charge returns every rechargeTicks. Charges and locks count the ticks time moves (none in a hit stop).

See `abilities §1`.

Also known as: skill, spell, cooldown, charges, ability slot, mana cost, Unreal GAS ability, special move

Token `og.Ability`, registered under the schema name `og.Ability`; its value type is `Ability`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `owner` | `Entity` | `0` | — | — | — | Who uses it, as og.AbilityUsed reports. |
| `action` | `string` | `""` | — | — | `str(32)`: at most 32 bytes of UTF-8 | The input action that uses it; empty: only og.abilities.use. |
| `maxCharges` | `number` | `0` | ≥ 0 | — | tunable | The charges it holds at most; 0 is never used. |
| `charges` | `number` | `0` | — | — | — | The charges it has: give its first ones when it is inserted; written as it is used and recharged. |
| `rechargeTicks` | `number` | `0` | ≥ 0 | `ticks` | tunable | Ticks for one charge to return while below maxCharges; 0 returns none. |
| `recharge` | `number` | `0` | — | `ticks` | read only | Written: ticks into the charge returning (0 when full). |
| `lockTicks` | `number` | `0` | ≥ 0 | `ticks` | tunable | Ticks after a use before it can be used again, whatever its charges; 0 locks nothing. |
| `lock` | `number` | `0` | — | `ticks` | read only | Written: ticks of its lock left. |
| `pool` | `Entity` | `0` | — | — | — | The og.Stat a use takes `cost` from (its base: mana, stamina, ammo); 0 costs nothing. |
| `cost` | `number` | `0` | ≥ 0 | — | tunable | What a use takes from the pool's base; a pool whose value is under it refuses the use. |

```ogt
component Ability {
  aliases "skill, spell, cooldown, charges, ability slot, mana cost, Unreal GAS ability, special move"
  doc "Something its owner uses: on a press of `action` (og.input.consume, taken only when it can be used, so an early press waits in the buffer) or by og.abilities.use, it spends a charge and its cost and starts its lock, and og.AbilityUsed is sent. Below maxCharges a charge returns every rechargeTicks. Charges and locks count the ticks time moves (none in a hit stop)."
  guide "abilities §1"
  field owner Entity "Who uses it, as og.AbilityUsed reports."
  field action str.32 "The input action that uses it; empty: only og.abilities.use."
  field maxCharges u32 "The charges it holds at most; 0 is never used." {
    min 0
    tunable true
  }
  field charges u32 "The charges it has: give its first ones when it is inserted; written as it is used and recharged."
  field rechargeTicks u32 "Ticks for one charge to return while below maxCharges; 0 returns none." {
    min 0
    unit ticks
    tunable true
  }
  field recharge u32 "Written: ticks into the charge returning (0 when full)." {
    readonly true
    unit ticks
  }
  field lockTicks u32 "Ticks after a use before it can be used again, whatever its charges; 0 locks nothing." {
    min 0
    unit ticks
    tunable true
  }
  field lock u32 "Written: ticks of its lock left." {
    readonly true
    unit ticks
  }
  field pool Entity "The og.Stat a use takes `cost` from (its base: mana, stamina, ammo); 0 costs nothing."
  field cost f32 "What a use takes from the pool's base; a pool whose value is under it refuses the use." {
    min 0
    tunable true
  }
}
```

<sub>Module `abilities` line 33</sub>
