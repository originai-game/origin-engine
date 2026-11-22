# abilities

og.systems.abilities on one hero (`src/main.ts`): a dash (KeyK) of two
charges, each back 60 ticks after it went, 12 ticks apart; a fireball
(KeyJ) of one charge, back after 90, that costs 20 of 50 mana; armour,
speed (boots: +1) and mana as og.Stat entities. The probe's commands apply
poison (stack, at most 3, -2 armour a stack), a slow (refresh, -50% speed)
and a shield (strongest, + its strength in armour).

What it checks (`tests/abilities.og.ts`, `og test tests/fixtures/abilities`):

- two dashes, a press inside the lock not taken; charges back on tick 90
  and tick 150, one at a time;
- a fireball that spends mana, is not taken without a charge, recharges and
  is refused once mana is under its cost;
- poison applied four times holds 3 stacks; a weaker shield is dropped and
  a stronger one replaces it; a refreshed slow ends 60 ticks after the
  refresh, poison 120 after its last;
- every tick, armour and speed are what the statuses on the hero make them
  and mana never goes below 0; two runs end on one hash.

Exact recharge ticks through a hit stop, a refused use leaving the world
hash unchanged, one stat value for every insertion order of its modifiers,
a tie under strongest, maxStacks 0 and a restore mid-recharge are
`packages/engine/tests/abilities.test.ts`.

Budget: under 10 s warm. No frames: nothing here is drawn.
