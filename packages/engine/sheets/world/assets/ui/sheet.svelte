<meta name="component" content="WorldSheet">

<!-- Every world label of the page and breakpoint in play, each kind a keyed
     collection of rows main.ts lays out from fixtures.ts: the third-person
     scene, the top-down scene and the two states boards. -->
<div id="Sheet">
  {#each $player as it (it.id)}<anchor id="PlayerProxy" at={it.at} reserve="player" reserve-radius={it.radius} reserve-height={it.height} />{/each}
  {#each $Nameplate as it (it.id)}<Nameplate entity={it.entity} at={it.at} height={it.height} name={it.name} title={it.title} level={it.level} reaction={it.reaction} health={it.health} shield={it.shield} castName={it.castName} castProgress={it.castProgress} castTime={it.castTime} castState={it.castState} icon={it.icon} elite={it.elite} boss={it.boss} targeted={it.targeted} dead={it.dead} lootable={it.lootable} compact={it.compact} maxDistance={it.maxDistance} />{/each}
  {#each $LootLabel as it (it.id)}<LootLabel entity={it.entity} at={it.at} height={it.height} name={it.name} count={it.count} rarity={it.rarity} hovered={it.hovered} action={it.action} more={it.more} tiers={it.tiers} />{/each}
  {#each $InteractionPrompt as it (it.id)}<InteractionPrompt entity={it.entity} at={it.at} height={it.height} action={it.action} key={it.key} pad={it.pad} verb={it.verb} name={it.name} pressed={it.pressed} hold={it.hold} disabled={it.disabled} reason={it.reason} maxDistance={it.maxDistance} />{/each}
  {#each $SpeechBubble as it (it.id)}<SpeechBubble entity={it.entity} at={it.at} height={it.height} speaker={it.speaker} reaction={it.reaction} text={it.text} shout={it.shout} left={it.left} />{/each}
  {#each $Telegraph as it (it.id)}<Telegraph entity={it.entity} at={it.at} shape={it.shape} radius={it.radius} inner={it.inner} angle={it.angle} width={it.width} direction={it.direction} progress={it.progress} friendly={it.friendly} hit={it.hit} />{/each}
  {#each $SelectionRing as it (it.id)}<SelectionRing entity={it.entity} at={it.at} radius={it.radius} side={it.side} leader={it.leader} target={it.target} hover={it.hover} />{/each}
  {#each $PlacementGhost as it (it.id)}<PlacementGhost at={it.at} width={it.width} depth={it.depth} rotation={it.rotation} columns={it.columns} cells={it.cells} valid={it.valid} label={it.label} range={it.range} />{/each}
  {#each $Waypoint as it (it.id)}<Waypoint entity={it.entity} at={it.at} height={it.height} kind={it.kind} label={it.label} distance={it.distance} units={it.units} range={it.range} edge={it.edge} />{/each}
  {#each $PathLine as it (it.id)}<PathLine at={it.at} size={it.size} path={it.path} reach={it.reach} stops={it.stops} endX={it.endX} endY={it.endY} label={it.label} invalid={it.invalid} />{/each}
  <OverheadBars bars={$bars} height={0} />
</div>

<style>
#Sheet { width: stretch; height: stretch; }
</style>
