// The sheet's fixture data (docs/ui.md §3.1: fixture values are data, not
// visual tuning): every fixture each member declares, as its document's
// <meta name="fixture:…"> has it, and the states the sheet adds (empty,
// overflow, disabled and the rest), each its default with what differs.
// Regenerate the documents' part when a document's fixture changes.

const abilityBar_default = {"column":false,"slots":[{"id":"q","icon":"sword","name":"Cleave","action":"","hotkey":"Q","cooldown":0,"cooldownMax":6,"cost":"","unaffordable":false,"locked":false,"active":false},{"id":"w","icon":"shield","name":"Guard","action":"","hotkey":"W","cooldown":3.4,"cooldownMax":10,"cost":"30","unaffordable":false,"locked":false,"active":false},{"id":"e","icon":"bolt","name":"Dash","action":"","hotkey":"E","cooldown":0,"cooldownMax":5,"cost":"60","unaffordable":true,"locked":false,"active":false},{"id":"r","icon":"fire","name":"Inferno","action":"","hotkey":"R","cooldown":0,"cooldownMax":60,"cost":"","unaffordable":false,"locked":true,"active":false},{"id":"t","icon":"potion","name":"Elixir","action":"","hotkey":"1","cooldown":0,"cooldownMax":0,"cost":"","unaffordable":false,"locked":false,"active":true}]};
const abilitySlot_default = {"icon":"fire","name":"Fireball","action":"","hotkey":"Q","cooldown":4.2,"cooldownMax":8,"charges":2,"maxCharges":3,"cost":"40","unaffordable":false,"outOfRange":false,"active":false,"locked":false,"disabled":false,"ultimate":-1,"level":2,"maxLevel":3};
const actionButton_default = {"action":"jump","label":"Jump","icon":"bolt","primary":true,"charge":0.7,"disabled":false};
const actionHints_default = {"label":"","hints":[{"action":"confirm","hint":"Buy","hold":false},{"action":"back","hint":"Back","hold":false},{"action":"details","hint":"Sell all","hold":true}]};
const breadcrumbs_default = {"label":"","items":[{"id":"home","label":"Home","icon":"home","current":false},{"id":"armory","label":"Armory","icon":"","current":false},{"id":"swords","label":"Swords","icon":"","current":false},{"id":"ember","label":"Ember Blade","icon":"","current":true}]};
const carousel_default = {"index":1,"label":"Season pass","pages":[{"id":"a","title":"Ember Knight","body":"A blade forged in the first fire.","image":0},{"id":"b","title":"Frost Warden","body":"Holds the northern gate.","image":0},{"id":"c","title":"Storm Caller","body":"Lightning answers her.","image":0}]};
const celebration_default = {"kind":"levelUp","shots":[{"id":"1"}],"title":"","value":"12"};
const chart_default = {"title":"Weekly kills","subtitle":"Last 7 days","max":100,"area":true,"line":"M24 112 L74 86 L122 99 L175 54 L224 68 L273 32 L322 43","ticks":[{"id":"0","at":0,"label":"0"},{"id":"50","at":0.5,"label":"50"},{"id":"100","at":1,"label":"100"}],"bars":[{"id":"mon","label":"Mon","value":32,"color":""},{"id":"tue","label":"Tue","value":51,"color":""},{"id":"wed","label":"Wed","value":44,"color":""},{"id":"thu","label":"Thu","value":70,"color":""},{"id":"fri","label":"Fri","value":62,"color":"#e5484d"},{"id":"sat","label":"Sat","value":88,"color":""},{"id":"sun","label":"Sun","value":79,"color":""}]};
const compass_default = {"heading":0.6,"units":"m","pois":[{"id":"quest","bearing":0.9,"distance":240,"kind":"objective","icon":"star","label":"Quest"},{"id":"camp","bearing":0.15,"distance":85,"kind":"poi","icon":"flag","label":"Camp"},{"id":"foe","bearing":1.6,"distance":-1,"kind":"enemy","icon":"","label":"Enemy"}]};
const cooldown_default = {"remaining":3.2,"total":8,"icon":"bolt","label":"Dash"};
const dataGrid_default = {"label":"Inventory","tracks":"2fr 1fr 64px 80px","selected":"b","loading":false,"error":"","empty":"","hint":"","columns":[{"id":"name","label":"Item","numeric":false},{"id":"kind","label":"Type","numeric":false},{"id":"qty","label":"Qty","numeric":true},{"id":"value","label":"Value","numeric":true}],"rows":[{"id":"a","cells":[{"id":"name","text":"Ember Blade","numeric":false},{"id":"kind","text":"Weapon","numeric":false},{"id":"qty","text":"1","numeric":true},{"id":"value","text":"1,250","numeric":true}]},{"id":"b","cells":[{"id":"name","text":"Health Potion","numeric":false},{"id":"kind","text":"Consumable","numeric":false},{"id":"qty","text":"12","numeric":true},{"id":"value","text":"40","numeric":true}]},{"id":"c","cells":[{"id":"name","text":"Iron Shield","numeric":false},{"id":"kind","text":"Armour","numeric":false},{"id":"qty","text":"1","numeric":true},{"id":"value","text":"380","numeric":true}]},{"id":"d","cells":[{"id":"name","text":"Silver Key","numeric":false},{"id":"kind","text":"Quest","numeric":false},{"id":"qty","text":"2","numeric":true},{"id":"value","text":"0","numeric":true}]},{"id":"e","cells":[{"id":"name","text":"Moonstone","numeric":false},{"id":"kind","text":"Gem","numeric":false},{"id":"qty","text":"3","numeric":true},{"id":"value","text":"900","numeric":true}]}]};
const heroButton_default = {"label":"End turn","tone":"","disabled":false,"key":"E"};
const holdButton_default = {"label":"Delete save","holding":"Keep holding…","hint":"Hold to delete","icon":"close","action":"","variant":"danger","disabled":false};
const hudTimer_default = {"seconds":8.4,"total":90,"countdown":true,"warnAt":10,"paused":false,"label":"Round 2"};
const menu_default = {"submenu":"","label":"","entries":[{"id":"h","label":"Item","kind":"header","icon":"","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"","more":false},{"id":"copy","label":"Copy","kind":"item","icon":"plus","shortcut":"Ctrl+C","checked":false,"danger":false,"disabled":false,"parent":"","more":false},{"id":"compare","label":"Compare","kind":"check","icon":"","shortcut":"C","checked":true,"danger":false,"disabled":false,"parent":"","more":false},{"id":"sort","label":"Sort by","kind":"item","icon":"flag","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"","more":true},{"id":"sell","label":"Sell","kind":"item","icon":"coin","shortcut":"","checked":false,"danger":false,"disabled":true,"parent":"","more":false},{"id":"s1","label":"","kind":"separator","icon":"","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"","more":false},{"id":"destroy","label":"Destroy","kind":"item","icon":"bomb","shortcut":"Del","checked":false,"danger":true,"disabled":false,"parent":"","more":false},{"id":"name","label":"Name","kind":"radio","icon":"","shortcut":"","checked":true,"danger":false,"disabled":false,"parent":"sort","more":false},{"id":"value","label":"Value","kind":"radio","icon":"","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"sort","more":false}]};
const minimapView_default = {"texture":0,"centerX":0.5,"centerY":0.5,"zoom":0.3,"heading":30,"rotate":false,"square":false,"zone":"Ashen Vale","markers":[{"id":"ally","kind":"ally","x":0.45,"y":0.42,"label":"Mira"},{"id":"foe","kind":"enemy","x":0.58,"y":0.55,"label":"Enemy"},{"id":"quest","kind":"objective","x":0.6,"y":0.38,"label":"Quest"},{"id":"ping","kind":"ping","x":0.42,"y":0.6,"label":"Ping"},{"id":"chest","kind":"loot","x":0.53,"y":0.47,"label":"Chest"}]};
const minimap_default = {"texture":0,"u0":0,"v0":0,"u1":1,"v1":1,"size":160,"label":"Ashen Vale","markers":[{"id":"me","x":0.5,"y":0.5,"tone":"neutral","player":true},{"id":"a","x":0.25,"y":0.3,"tone":"danger","player":false},{"id":"b","x":0.72,"y":0.65,"tone":"success","player":false},{"id":"c","x":0.6,"y":0.2,"tone":"warn","player":false}]};
const nodeGraph_default = {"view":"0 0 360 240","zoom":1,"panX":0,"panY":0,"selected":"cleave","name":"","links":[{"id":"a","d":"M40 120 C 100 120, 120 60, 180 60","active":true},{"id":"b","d":"M40 120 C 100 120, 120 180, 180 180","active":false},{"id":"c","d":"M180 60 L 320 120","active":false}],"nodes":[{"id":"root","x":40,"y":120,"label":"Strength","icon":"sword","state":"done","shape":"circle","team":""},{"id":"cleave","x":180,"y":60,"label":"Cleave","icon":"bolt","state":"available","shape":"hex","team":""},{"id":"guard","x":180,"y":180,"label":"Guard","icon":"shield","state":"active","shape":"square","team":"ally"},{"id":"rage","x":320,"y":120,"label":"Rage","icon":"fire","state":"locked","shape":"diamond","team":"enemy"}]};
const orbMeter_default = {"value":340,"max":500,"kind":"health","showValue":true,"label":"Life","name":""};
const overlayBar_default = {"selected":"heat","label":"","overlays":[{"id":"heat","name":"Heat","icon":"fire","image":0,"ramp":"#2b6cb0, #f6e05e, #e53e3e","low":"Cold","high":"Hot"},{"id":"water","name":"Water","icon":"drop","image":0,"ramp":"#d9e8f5, #2b6cb0","low":"Dry","high":"Flooded"},{"id":"crime","name":"Crime","icon":"skull","image":0,"ramp":"#38a169, #e53e3e","low":"Safe","high":"Dangerous"},{"id":"land","name":"Land value","icon":"coin","image":0,"ramp":"#4a5568, #ecc94b","low":"Low","high":"High"}]};
const progressBar_default = {"value":0.42,"buffer":0.6,"label":"Downloading","valueText":"","showValue":true,"indeterminate":false,"tone":""};
const progressRing_default = {"value":0.62,"spinning":false,"label":"","name":"","tone":""};
const radialMenu_default = {"span":0.1667,"chosen":"heal","detail":"Restore 40 health","items":[{"id":"heal","label":"Heal","icon":"heart","at":0,"cost":"","unaffordable":false,"disabled":false},{"id":"shield","label":"Ward","icon":"shield","at":0.1667,"cost":"150","unaffordable":false,"disabled":false},{"id":"bomb","label":"Bomb","icon":"bomb","at":0.3333,"cost":"300","unaffordable":true,"disabled":false},{"id":"map","label":"Map","icon":"map","at":0.5,"cost":"","unaffordable":false,"disabled":false},{"id":"bag","label":"Bag","icon":"bag","at":0.6667,"cost":"","unaffordable":false,"disabled":true},{"id":"flag","label":"Rally","icon":"flag","at":0.8333,"cost":"","unaffordable":false,"disabled":false}]};
const radialMeter_default = {"value":1.4,"wheels":2,"exhausted":false,"hidden":false,"name":""};
const ribbon_default = {"text":"New","corner":"","tone":"danger"};
const skeletonGroup_default = {"shape":"row","items":[{"id":"a"},{"id":"b"},{"id":"c"}],"name":""};
const spinner_default = {"size":28,"name":""};
const splitView_default = {"ratio":0.35,"column":false,"label":""};
const starBar_default = {"value":2140,"max":3000,"stars":[{"id":"1","at":0.3333},{"id":"2","at":0.6667},{"id":"3","at":1}],"label":"Score","valueText":"","compact":false};
const statBar_default = {"value":72,"max":100,"kind":"health","segment":25,"shield":60,"shieldMax":100,"heal":12,"recoverable":0,"label":"HP","icon":"heart","showValue":true,"valueInside":false,"valueText":"","low":0.3,"critical":0.15,"bare":false,"name":""};
const statusGrid_default = {"more":2,"buffs":[{"id":"haste","icon":"bolt","name":"Haste","stacks":0,"remaining":12,"duration":30,"expiring":false},{"id":"shield","icon":"shield","name":"Barrier","stacks":3,"remaining":4.2,"duration":10,"expiring":true},{"id":"regen","icon":"heart","name":"Regrowth","stacks":0,"remaining":0,"duration":0,"expiring":false}],"debuffs":[{"id":"burn","icon":"fire","name":"Burning","stacks":2,"remaining":6,"duration":8,"expiring":false,"type":"magic"},{"id":"venom","icon":"drop","name":"Venom","stacks":0,"remaining":18,"duration":20,"expiring":false,"type":"poison"}]};
const tooltipCard_default = {"title":"Ember Blade","subtitle":"Legendary sword","rarity":"legendary","icon":"sword","image":0,"body":"Burns on hit for 12 fire damage over 3 seconds. Two-handed.","stats":[{"id":"dmg","label":"Damage","value":"42–58","delta":12,"deltaText":"+12","better":false},{"id":"speed","label":"Attack speed","value":"1.1","delta":-0.2,"deltaText":"−0.2","better":false},{"id":"cd","label":"Cooldown","value":"4 s","delta":0,"deltaText":"","better":true}],"compareTitle":"Equipped","compareStats":[{"id":"dmg","label":"Damage","value":"30–46"},{"id":"speed","label":"Attack speed","value":"1.3"},{"id":"cd","label":"Cooldown","value":"4 s"}],"footer":"Sell for 120","hints":[{"id":"equip","text":"Equip","action":"confirm"}]};
const touchControls_default = {"move":"move","pad":false,"look":"","deadZone":0,"always":true,"disabled":false,"actions":[{"id":"jump","action":"jump","label":"Jump","icon":"bolt","primary":true,"charge":-1,"disabled":false},{"id":"dash","action":"dash","label":"Dash","icon":"fire","primary":false,"charge":-1,"disabled":false},{"id":"use","action":"use","label":"Use","icon":"key","primary":false,"charge":-1,"disabled":false}]};
const touchControls_pad = {"move":"move","pad":true,"look":"","deadZone":0,"always":true,"disabled":false,"actions":[{"id":"a","action":"confirm","label":"A","icon":"","primary":true,"charge":-1,"disabled":false},{"id":"b","action":"back","label":"B","icon":"","primary":false,"charge":-1,"disabled":false}]};
const touchControls_twin = {"move":"move","pad":false,"look":"look","deadZone":0.2,"always":true,"disabled":false,"actions":[{"id":"fire","action":"fire","label":"Fire","icon":"bomb","primary":true,"charge":-1,"disabled":false},{"id":"ult","action":"ultimate","label":"Ultimate","icon":"star","primary":false,"charge":0.6,"disabled":true}]};
const touchControls_many = {"move":"move","pad":false,"look":"","deadZone":0,"always":true,"disabled":false,"actions":[{"id":"attack","action":"attack","label":"Attack","icon":"sword","primary":true,"charge":-1,"disabled":false},{"id":"block","action":"block","label":"Block","icon":"shield","primary":false,"charge":-1,"disabled":false},{"id":"dodge","action":"dodge","label":"Dodge","icon":"bolt","primary":false,"charge":-1,"disabled":false},{"id":"heal","action":"heal","label":"Heal","icon":"potion","primary":false,"charge":-1,"disabled":false},{"id":"bag","action":"bag","label":"Bag","icon":"bag","primary":false,"charge":-1,"disabled":false}]};
const touchControls_disabled = {"move":"move","pad":false,"look":"","deadZone":0,"always":true,"disabled":true,"actions":[{"id":"jump","action":"jump","label":"Jump","icon":"bolt","primary":true,"charge":-1,"disabled":false}]};
const touchControls_empty = {"move":"","pad":false,"look":"","deadZone":0,"always":true,"disabled":false,"actions":[]};

export const FIXTURES: Record<string, Record<string, Record<string, unknown>>> = {
  abilityBar: {
    default: abilityBar_default,
    empty: { ...abilityBar_default, "slots": [] },
    column: { ...abilityBar_default, "column": true },
    overflow: { ...abilityBar_default, "slots": [{"id":"s0","icon":"sword","name":"Cleave of the Ninefold Gate","action":"","hotkey":"Q","cooldown":0,"cooldownMax":6,"cost":"","unaffordable":false,"locked":false,"active":false},{"id":"s1","icon":"shield","name":"Guard of the Ninefold Gate","action":"","hotkey":"W","cooldown":3.4,"cooldownMax":10,"cost":"30","unaffordable":false,"locked":false,"active":false},{"id":"s2","icon":"bolt","name":"Dash of the Ninefold Gate","action":"","hotkey":"E","cooldown":59.5,"cooldownMax":5,"cost":"60","unaffordable":true,"locked":false,"active":false},{"id":"s3","icon":"fire","name":"Inferno of the Ninefold Gate","action":"","hotkey":"R","cooldown":0,"cooldownMax":60,"cost":"","unaffordable":false,"locked":true,"active":false},{"id":"s4","icon":"potion","name":"Elixir of the Ninefold Gate","action":"","hotkey":"1","cooldown":0,"cooldownMax":0,"cost":"","unaffordable":false,"locked":false,"active":true},{"id":"s5","icon":"sword","name":"Cleave of the Ninefold Gate","action":"","hotkey":"2","cooldown":1.2,"cooldownMax":6,"cost":"","unaffordable":false,"locked":false,"active":false},{"id":"s6","icon":"shield","name":"Guard of the Ninefold Gate","action":"","hotkey":"3","cooldown":0,"cooldownMax":10,"cost":"30","unaffordable":false,"locked":false,"active":false},{"id":"s7","icon":"bolt","name":"Dash of the Ninefold Gate","action":"","hotkey":"4","cooldown":0,"cooldownMax":5,"cost":"60","unaffordable":true,"locked":false,"active":false},{"id":"s8","icon":"fire","name":"Inferno of the Ninefold Gate","action":"","hotkey":"Shift+F","cooldown":0,"cooldownMax":60,"cost":"","unaffordable":false,"locked":true,"active":false},{"id":"s9","icon":"potion","name":"Elixir of the Ninefold Gate","action":"","hotkey":"Ctrl+5","cooldown":12.8,"cooldownMax":0,"cost":"","unaffordable":false,"locked":false,"active":true},{"id":"s10","icon":"sword","name":"Cleave of the Ninefold Gate","action":"","hotkey":"F","cooldown":0,"cooldownMax":6,"cost":"","unaffordable":false,"locked":false,"active":false},{"id":"s11","icon":"shield","name":"Guard of the Ninefold Gate","action":"","hotkey":"G","cooldown":0,"cooldownMax":10,"cost":"30","unaffordable":false,"locked":false,"active":false}] },
  },
  abilitySlot: {
    default: abilitySlot_default,
    ready: { ...abilitySlot_default, "cooldown": 0, "charges": 3 },
    unaffordable: { ...abilitySlot_default, "cooldown": 0, "unaffordable": true },
    outOfRange: { ...abilitySlot_default, "cooldown": 0, "outOfRange": true },
    active: { ...abilitySlot_default, "cooldown": 0, "active": true },
    locked: { ...abilitySlot_default, "cooldown": 0, "locked": true },
    disabled: { ...abilitySlot_default, "cooldown": 0, "disabled": true },
    ultimate: { ...abilitySlot_default, "cooldown": 0, "charges": 0, "maxCharges": 0, "ultimate": 0.72 },
    ultimateReady: { ...abilitySlot_default, "cooldown": 0, "charges": 0, "maxCharges": 0, "ultimate": 1 },
    overflow: { ...abilitySlot_default, "name": "Ysolde the Unbroken, Warden of the Ninefold Gate", "hotkey": "Ctrl+Shift+5", "cost": "12,500", "cooldown": 3599 },
  },
  actionButton: {
    default: actionButton_default,
    secondary: { ...actionButton_default, "primary": false, "charge": -1 },
    disabled: { ...actionButton_default, "disabled": true },
    noLabel: { ...actionButton_default, "label": "", "charge": -1 },
  },
  actionHints: {
    default: actionHints_default,
    empty: { ...actionHints_default, "hints": [] },
    overflow: { ...actionHints_default, "hints": [{"action":"confirm","hint":"Buy and equip immediately","hold":false},{"action":"back","hint":"Back to the market","hold":false},{"action":"details","hint":"Sell every duplicate","hold":true},{"action":"menu","hint":"Open the menu","hold":false},{"action":"next","hint":"Next tab","hold":false},{"action":"previous","hint":"Previous tab","hold":true}] },
  },
  breadcrumbs: {
    default: breadcrumbs_default,
    single: { ...breadcrumbs_default, "items": [{"id":"home","label":"Home","icon":"home","current":true}] },
    overflow: { ...breadcrumbs_default, "items": [{"id":"b0","label":"Home","icon":"home","current":false},{"id":"b1","label":"Armory of the Ninefold Gate","icon":"","current":false},{"id":"b2","label":"Two-handed weapons","icon":"","current":false},{"id":"b3","label":"Swords","icon":"","current":false},{"id":"b4","label":"Legendary","icon":"","current":false},{"id":"b5","label":"Ember Blade, the Burning Oath","icon":"","current":false},{"id":"b6","label":"Compare","icon":"","current":true}] },
  },
  carousel: {
    default: carousel_default,
    first: { ...carousel_default, "index": 0 },
    single: { ...carousel_default, "index": 0, "pages": [{"id":"a","title":"Ember Knight","body":"A blade forged in the first fire.","image":0}] },
    empty: { ...carousel_default, "index": 0, "pages": [] },
    overflow: { ...carousel_default, "label": "Season pass · Chapter of the Ninefold Gate", "pages": [{"id":"p0","title":"Ysolde the Unbroken, Warden of the Ninefold Gate","body":"A blade forged in the first fire, carried through every siege of the eastern marches and never once surrendered.","image":0},{"id":"p1","title":"Ysolde the Unbroken, Warden of the Ninefold Gate","body":"A blade forged in the first fire, carried through every siege of the eastern marches and never once surrendered.","image":0},{"id":"p2","title":"Ysolde the Unbroken, Warden of the Ninefold Gate","body":"A blade forged in the first fire, carried through every siege of the eastern marches and never once surrendered.","image":0}] },
  },
  celebration: {
    default: celebration_default,
    confetti: { ...celebration_default, "kind": "confetti", "value": "" },
    burst: { ...celebration_default, "kind": "burst", "value": "" },
    fireworks: { ...celebration_default, "kind": "fireworks", "value": "" },
  },
  chart: {
    default: chart_default,
    empty: { ...chart_default, "bars": [], "line": "", "area": false },
    bars: { ...chart_default, "line": "", "area": false },
    overflow: { ...chart_default, "title": "Weekly kills across every region of the Ninefold Gate", "subtitle": "Last 14 days, all modes", "bars": [{"id":"d0","label":"Day 1","value":32,"color":""},{"id":"d1","label":"Day 2","value":51,"color":""},{"id":"d2","label":"Day 3","value":44,"color":""},{"id":"d3","label":"Day 4","value":70,"color":""},{"id":"d4","label":"Day 5","value":62,"color":"#e5484d"},{"id":"d5","label":"Day 6","value":88,"color":""},{"id":"d6","label":"Day 7","value":79,"color":""},{"id":"d7","label":"Day 8","value":100,"color":""},{"id":"d8","label":"Day 9","value":12,"color":""},{"id":"d9","label":"Day 10","value":0,"color":""},{"id":"d10","label":"Day 11","value":95,"color":""},{"id":"d11","label":"Day 12","value":64,"color":"#e5484d"},{"id":"d12","label":"Day 13","value":37,"color":""},{"id":"d13","label":"Day 14","value":81,"color":""}] },
  },
  compass: {
    default: compass_default,
    empty: { ...compass_default, "pois": [] },
    overflow: { ...compass_default, "pois": [{"id":"c0","bearing":0.5,"distance":240,"kind":"objective","icon":"star","label":"Quest"},{"id":"c1","bearing":0.54,"distance":85,"kind":"poi","icon":"flag","label":"Camp"},{"id":"c2","bearing":0.58,"distance":-1,"kind":"enemy","icon":"","label":"Enemy"},{"id":"c3","bearing":0.62,"distance":12400,"kind":"objective","icon":"star","label":"The Lighthouse of the Northern Reach"},{"id":"c4","bearing":0.66,"distance":3,"kind":"poi","icon":"flag","label":"Ally"},{"id":"c5","bearing":0.7,"distance":999,"kind":"enemy","icon":"","label":"Ping"},{"id":"c6","bearing":0.74,"distance":40,"kind":"objective","icon":"star","label":"Chest"},{"id":"c7","bearing":0.78,"distance":1,"kind":"poi","icon":"flag","label":"Gate"}] },
  },
  cooldown: {
    default: cooldown_default,
    ready: { ...cooldown_default, "remaining": 0 },
    almost: { ...cooldown_default, "remaining": 0.4 },
    long: { ...cooldown_default, "remaining": 3599, "total": 3600, "label": "Ritual of the Ninefold Gate" },
  },
  dataGrid: {
    default: dataGrid_default,
    loading: { ...dataGrid_default, "loading": true, "rows": [] },
    error: { ...dataGrid_default, "error": "The market could not be reached. Try again in a moment.", "rows": [] },
    empty: { ...dataGrid_default, "empty": "Your inventory is empty", "rows": [], "selected": "" },
    overflow: { ...dataGrid_default, "rows": [{"id":"r0","cells":[{"id":"name","text":"Ysolde the Unbroken, Warden of the Ninefold Gate","numeric":false},{"id":"kind","text":"Weapon","numeric":false},{"id":"qty","text":"1","numeric":true},{"id":"value","text":"1,250","numeric":true}]},{"id":"r1","cells":[{"id":"name","text":"Health Potion","numeric":false},{"id":"kind","text":"Consumable","numeric":false},{"id":"qty","text":"12","numeric":true},{"id":"value","text":"1,234,567","numeric":true}]},{"id":"r2","cells":[{"id":"name","text":"Iron Shield","numeric":false},{"id":"kind","text":"Armour","numeric":false},{"id":"qty","text":"1","numeric":true},{"id":"value","text":"380","numeric":true}]},{"id":"r3","cells":[{"id":"name","text":"Ysolde the Unbroken, Warden of the Ninefold Gate","numeric":false},{"id":"kind","text":"Quest","numeric":false},{"id":"qty","text":"2","numeric":true},{"id":"value","text":"0","numeric":true}]},{"id":"r4","cells":[{"id":"name","text":"Moonstone","numeric":false},{"id":"kind","text":"Gem","numeric":false},{"id":"qty","text":"3","numeric":true},{"id":"value","text":"900","numeric":true}]},{"id":"r5","cells":[{"id":"name","text":"Ember Blade","numeric":false},{"id":"kind","text":"Weapon","numeric":false},{"id":"qty","text":"1","numeric":true},{"id":"value","text":"1,234,567","numeric":true}]},{"id":"r6","cells":[{"id":"name","text":"Ysolde the Unbroken, Warden of the Ninefold Gate","numeric":false},{"id":"kind","text":"Consumable","numeric":false},{"id":"qty","text":"12","numeric":true},{"id":"value","text":"40","numeric":true}]},{"id":"r7","cells":[{"id":"name","text":"Iron Shield","numeric":false},{"id":"kind","text":"Armour","numeric":false},{"id":"qty","text":"1","numeric":true},{"id":"value","text":"380","numeric":true}]},{"id":"r8","cells":[{"id":"name","text":"Silver Key","numeric":false},{"id":"kind","text":"Quest","numeric":false},{"id":"qty","text":"2","numeric":true},{"id":"value","text":"0","numeric":true}]},{"id":"r9","cells":[{"id":"name","text":"Ysolde the Unbroken, Warden of the Ninefold Gate","numeric":false},{"id":"kind","text":"Gem","numeric":false},{"id":"qty","text":"3","numeric":true},{"id":"value","text":"1,234,567","numeric":true}]},{"id":"r10","cells":[{"id":"name","text":"Ember Blade","numeric":false},{"id":"kind","text":"Weapon","numeric":false},{"id":"qty","text":"1","numeric":true},{"id":"value","text":"1,250","numeric":true}]},{"id":"r11","cells":[{"id":"name","text":"Health Potion","numeric":false},{"id":"kind","text":"Consumable","numeric":false},{"id":"qty","text":"12","numeric":true},{"id":"value","text":"40","numeric":true}]},{"id":"r12","cells":[{"id":"name","text":"Ysolde the Unbroken, Warden of the Ninefold Gate","numeric":false},{"id":"kind","text":"Armour","numeric":false},{"id":"qty","text":"1","numeric":true},{"id":"value","text":"380","numeric":true}]},{"id":"r13","cells":[{"id":"name","text":"Silver Key","numeric":false},{"id":"kind","text":"Quest","numeric":false},{"id":"qty","text":"2","numeric":true},{"id":"value","text":"1,234,567","numeric":true}]}], "selected": "r7", "hint": "Sorted by value" },
  },
  heroButton: {
    default: heroButton_default,
    premium: { ...heroButton_default, "tone": "premium" },
    secondary: { ...heroButton_default, "tone": "secondary" },
    danger: { ...heroButton_default, "tone": "danger", "label": "Abandon quest" },
    disabled: { ...heroButton_default, "disabled": true },
    overflow: { ...heroButton_default, "label": "End the turn and begin the siege of the Ninefold Gate", "key": "Ctrl+Enter" },
  },
  holdButton: {
    default: holdButton_default,
    primary: { ...holdButton_default, "variant": "primary", "label": "Claim rewards", "icon": "check", "hint": "Hold to claim" },
    secondary: { ...holdButton_default, "variant": "secondary", "label": "Skip", "icon": "play", "hint": "Hold to skip" },
    disabled: { ...holdButton_default, "disabled": true },
  },
  hudTimer: {
    default: hudTimer_default,
    running: { ...hudTimer_default, "seconds": 72 },
    paused: { ...hudTimer_default, "seconds": 72, "paused": true },
    countUp: { ...hudTimer_default, "countdown": false, "seconds": 3725, "total": 0, "label": "" },
    zero: { ...hudTimer_default, "seconds": 0 },
  },
  menu: {
    default: menu_default,
    submenu: { ...menu_default, "submenu": "sort" },
    empty: { ...menu_default, "entries": [] },
  },
  minimapView: {
    default: minimapView_default,
    square: { ...minimapView_default, "square": true, "rotate": true },
    empty: { ...minimapView_default, "markers": [], "zone": "" },
  },
  minimap: {
    default: minimap_default,
    empty: { ...minimap_default, "markers": [], "label": "" },
    small: { ...minimap_default, "size": 96 },
  },
  nodeGraph: {
    default: nodeGraph_default,
    empty: { ...nodeGraph_default, "nodes": [], "links": [], "selected": "" },
    zoomed: { ...nodeGraph_default, "zoom": 1.6, "panX": -60, "panY": -20 },
  },
  orbMeter: {
    default: orbMeter_default,
    mana: { ...orbMeter_default, "kind": "mana", "value": 120, "max": 300, "label": "Mana" },
    empty: { ...orbMeter_default, "value": 0 },
    full: { ...orbMeter_default, "value": 500 },
    noValue: { ...orbMeter_default, "showValue": false },
  },
  overlayBar: {
    default: overlayBar_default,
    none: { ...overlayBar_default, "selected": "" },
    empty: { ...overlayBar_default, "overlays": [], "selected": "" },
  },
  progressBar: {
    default: progressBar_default,
    empty: { ...progressBar_default, "value": 0, "buffer": 0 },
    full: { ...progressBar_default, "value": 1, "buffer": 1, "tone": "success", "label": "Installed" },
    indeterminate: { ...progressBar_default, "indeterminate": true },
    warn: { ...progressBar_default, "tone": "warn" },
    danger: { ...progressBar_default, "tone": "danger", "value": 0.12, "label": "Connection unstable" },
    overflow: { ...progressBar_default, "label": "Downloading the Ninefold Gate expansion and its voice packs", "valueText": "12.4 GB of 31.0 GB · 42 minutes left" },
  },
  progressRing: {
    default: progressRing_default,
    spinning: { ...progressRing_default, "spinning": true },
    full: { ...progressRing_default, "value": 1, "tone": "success" },
    empty: { ...progressRing_default, "value": 0 },
    labelled: { ...progressRing_default, "label": "62%" },
  },
  radialMenu: {
    default: radialMenu_default,
    empty: { ...radialMenu_default, "items": [], "chosen": "", "detail": "" },
    few: { ...radialMenu_default, "span": 0.3333, "items": [{"id":"heal","label":"Heal","icon":"heart","at":0.0,"cost":"","unaffordable":false,"disabled":false},{"id":"shield","label":"Ward","icon":"shield","at":0.3333333333333333,"cost":"150","unaffordable":false,"disabled":false},{"id":"bomb","label":"Bomb","icon":"bomb","at":0.6666666666666666,"cost":"300","unaffordable":true,"disabled":false}] },
  },
  radialMeter: {
    default: radialMeter_default,
    full: { ...radialMeter_default, "value": 2 },
    exhausted: { ...radialMeter_default, "value": 0.3, "exhausted": true },
    empty: { ...radialMeter_default, "value": 0 },
  },
  ribbon: {
    default: ribbon_default,
    topLeft: { ...ribbon_default, "corner": "topLeft", "tone": "success", "text": "Sale" },
    bottomRight: { ...ribbon_default, "corner": "bottomRight", "tone": "warn", "text": "−50%" },
    stamp: { ...ribbon_default, "corner": "center", "tone": "primary", "text": "Sold out" },
    overflow: { ...ribbon_default, "text": "Limited edition of the Ninefold Gate" },
  },
  skeletonGroup: {
    default: skeletonGroup_default,
    lines: { ...skeletonGroup_default, "shape": "lines" },
    avatar: { ...skeletonGroup_default, "shape": "avatar" },
    block: { ...skeletonGroup_default, "shape": "block" },
    card: { ...skeletonGroup_default, "shape": "card" },
  },
  spinner: {
    default: spinner_default,
    large: { ...spinner_default, "size": 64 },
  },
  splitView: {
    default: splitView_default,
    column: { ...splitView_default, "column": true, "ratio": 0.6 },
  },
  starBar: {
    default: starBar_default,
    empty: { ...starBar_default, "value": 0 },
    full: { ...starBar_default, "value": 3000 },
    compact: { ...starBar_default, "compact": true },
  },
  statBar: {
    default: statBar_default,
    low: { ...statBar_default, "value": 26, "shield": 0, "heal": 0 },
    critical: { ...statBar_default, "value": 9, "shield": 0, "heal": 0 },
    empty: { ...statBar_default, "value": 0, "shield": 0, "heal": 0 },
    mana: { ...statBar_default, "kind": "mana", "value": 140, "max": 200, "segment": 0, "shield": 0, "heal": 0, "label": "MP", "icon": "drop" },
    stamina: { ...statBar_default, "kind": "stamina", "value": 40, "segment": 0, "shield": 0, "heal": 0, "label": "SP", "icon": "bolt", "valueInside": true },
    xp: { ...statBar_default, "kind": "xp", "value": 12840, "max": 20000, "segment": 0, "shield": 0, "heal": 0, "label": "XP", "icon": "star", "recoverable": 0 },
    recoverable: { ...statBar_default, "value": 48, "recoverable": 70, "shield": 0, "heal": 0 },
    overflow: { ...statBar_default, "value": 1284000, "max": 2000000, "segment": 0, "shield": 0, "heal": 0, "label": "Ysolde the Unbroken, Warden of the Ninefold Gate", "valueText": "1,284,000 / 2,000,000" },
  },
  statusGrid: {
    default: statusGrid_default,
    empty: { ...statusGrid_default, "buffs": [], "debuffs": [], "more": 0 },
    overflow: { ...statusGrid_default, "more": 14, "buffs": [{"id":"b0","icon":"bolt","name":"Haste","stacks":0,"remaining":12,"duration":30,"expiring":false},{"id":"b1","icon":"shield","name":"Barrier","stacks":3,"remaining":4.2,"duration":10,"expiring":true},{"id":"b2","icon":"heart","name":"Regrowth","stacks":99,"remaining":0,"duration":0,"expiring":false},{"id":"b3","icon":"bolt","name":"Haste","stacks":0,"remaining":12,"duration":30,"expiring":false},{"id":"b4","icon":"shield","name":"Barrier","stacks":12,"remaining":4.2,"duration":10,"expiring":true},{"id":"b5","icon":"heart","name":"Regrowth","stacks":0,"remaining":0,"duration":0,"expiring":false},{"id":"b6","icon":"bolt","name":"Haste","stacks":0,"remaining":12,"duration":30,"expiring":false},{"id":"b7","icon":"shield","name":"Barrier","stacks":2,"remaining":4.2,"duration":10,"expiring":true},{"id":"b8","icon":"heart","name":"Regrowth","stacks":0,"remaining":0,"duration":0,"expiring":false}], "debuffs": [{"id":"d0","icon":"fire","name":"Burning","stacks":2,"remaining":6,"duration":8,"expiring":false,"type":"magic"},{"id":"d1","icon":"drop","name":"Venom","stacks":0,"remaining":18,"duration":20,"expiring":false,"type":"poison"},{"id":"d2","icon":"fire","name":"Burning","stacks":2,"remaining":3599,"duration":3600,"expiring":false,"type":"magic"},{"id":"d3","icon":"drop","name":"Venom","stacks":0,"remaining":0.4,"duration":20,"expiring":false,"type":"poison"},{"id":"d4","icon":"fire","name":"Burning","stacks":2,"remaining":2,"duration":8,"expiring":false,"type":"magic"},{"id":"d5","icon":"drop","name":"Venom","stacks":0,"remaining":9,"duration":20,"expiring":false,"type":"poison"},{"id":"d6","icon":"fire","name":"Burning","stacks":2,"remaining":1,"duration":8,"expiring":false,"type":"magic"}] },
  },
  tooltipCard: {
    default: tooltipCard_default,
    plain: { ...tooltipCard_default, "rarity": "common", "subtitle": "", "stats": [], "compareTitle": "", "compareStats": [], "footer": "", "hints": [], "body": "A plain iron sword." },
    overflow: { ...tooltipCard_default, "title": "Ysolde the Unbroken, Warden of the Ninefold Gate", "subtitle": "Mythic two-handed sword of the eastern marches", "rarity": "mythic", "body": "Burns on hit for 12 fire damage over 3 seconds. Every third hit sets the ground alight for 4 seconds, and allies standing in the flames are healed for half the damage they deal. Two-handed.", "stats": [{"id":"s0","label":"Damage","value":"42–58","delta":12,"deltaText":"+12","better":false},{"id":"s1","label":"Attack speed","value":"1.1","delta":-0.2,"deltaText":"−0.2","better":false},{"id":"s2","label":"Cooldown","value":"4 s","delta":0,"deltaText":"","better":true},{"id":"s3","label":"Critical strike chance against burning enemies","value":"42–58","delta":12,"deltaText":"+12","better":false},{"id":"s4","label":"Range","value":"1.1","delta":-0.2,"deltaText":"−0.2","better":false},{"id":"s5","label":"Weight","value":"4 s","delta":0,"deltaText":"","better":true},{"id":"s6","label":"Durability","value":"42–58","delta":12,"deltaText":"+12","better":false},{"id":"s7","label":"Level","value":"1.1","delta":-0.2,"deltaText":"−0.2","better":false}] },
  },
  touchControls: {
    default: touchControls_default,
    pad: touchControls_pad,
    twin: touchControls_twin,
    many: touchControls_many,
    disabled: touchControls_disabled,
    empty: touchControls_empty,
  },
};

export const NAMES: Record<string, string> = {"abilityBar":"AbilityBar","abilitySlot":"AbilitySlot","actionButton":"ActionButton","actionHints":"ActionHints","breadcrumbs":"Breadcrumbs","carousel":"Carousel","celebration":"Celebration","chart":"Chart","compass":"Compass","cooldown":"Cooldown","dataGrid":"DataGrid","heroButton":"HeroButton","holdButton":"HoldButton","hudTimer":"HudTimer","menu":"Menu","minimapView":"MinimapView","minimap":"Minimap","nodeGraph":"NodeGraph","orbMeter":"OrbMeter","overlayBar":"OverlayBar","progressBar":"ProgressBar","progressRing":"ProgressRing","radialMenu":"RadialMenu","radialMeter":"RadialMeter","ribbon":"Ribbon","skeletonGroup":"SkeletonGroup","spinner":"Spinner","splitView":"SplitView","starBar":"StarBar","statBar":"StatBar","statusGrid":"StatusGrid","tooltipCard":"TooltipCard","touchControls":"TouchControls"};
