// The world sheet's fixture data (docs/ui.md §3.1): each world-anchored
// component's document fixture (`default`) and the states the sheet adds,
// each placed on the sheet's ground by `at` (x y z, world units; the units
// stand at src/main.ts's UNITS). Kept equal to the documents' fixtures by hand.

const LONG = "Ysolde the Unbroken, Warden of the Ninefold Gate";

export const FIXTURES: Record<string, Record<string, Record<string, unknown>>> = {
  nameplate: {
    default: {"entity":0,"at":"-4 0 -1","height":2.1,"name":"Grimtooth","title":"Warlord","level":"32","reaction":"hostile","health":0.62,"shield":0.15,"castName":"Cleave","castProgress":0.4,"icon":"skull","boss":true,"targeted":true,"maxDistance":80},
    friendly: {"entity":0,"at":"3 0 -2","height":2.1,"name":"Mira","title":"","level":"","reaction":"friendly","health":0.86,"shield":0,"castName":"","castProgress":0,"icon":"heart","boss":false,"targeted":false,"maxDistance":80},
    player: {"entity":0,"at":"0 0 2","height":2.1,"name":"Rhea","title":"","level":"31","reaction":"player","health":0.48,"shield":0,"castName":"","castProgress":0,"icon":"","boss":false,"targeted":false,"maxDistance":80},
    neutral: {"entity":0,"at":"7 0 -4","height":2.1,"name":"Old Tamsin","title":"Merchant","level":"","reaction":"","health":-1,"shield":0,"castName":"","castProgress":0,"icon":"","boss":false,"targeted":false,"maxDistance":80},
    overflow: {"entity":0,"at":"-2 0 -3","height":2.1,"name":LONG,"title":"Guardian of the Eastern Marches and the Ninefold Gate","level":"120","reaction":"hostile","health":0.62,"shield":0.15,"castName":"Ritual of the Ninefold Gate","castProgress":0.85,"icon":"skull","boss":true,"targeted":false,"maxDistance":80},
    cluster: {"entity":0,"at":"-1.4 0 -3.2","height":2.1,"name":"Ashen Pyre","title":"","level":"44","reaction":"hostile","health":0.12,"shield":0,"castName":"","castProgress":0,"icon":"","boss":false,"targeted":false,"maxDistance":80},
    far: {"entity":0,"at":"2 0 -30","height":2.1,"name":"Distant Raider","title":"","level":"","reaction":"hostile","health":0.9,"shield":0,"castName":"","castProgress":0,"icon":"","boss":false,"targeted":false,"maxDistance":20},
  },
  lootLabel: {
    default: {"entity":0,"at":"-6 0 2.5","height":0.4,"name":"Ember Blade","count":1,"rarity":"epic"},
    common: {"entity":0,"at":"-7.5 0 3.5","height":0.4,"name":"Iron Ore","count":12,"rarity":""},
    uncommon: {"entity":0,"at":"-5 0 4","height":0.4,"name":"Moonleaf","count":3,"rarity":"uncommon"},
    rare: {"entity":0,"at":"5 0 3.5","height":0.4,"name":"Silver Key","count":1,"rarity":"rare"},
    legendary: {"entity":0,"at":"6.5 0 2.5","height":0.4,"name":"Crown of Ash","count":1,"rarity":"legendary"},
    overflow: {"entity":0,"at":"5.5 0 1","height":0.4,"name":"Ember Blade of the Ninefold Gate, the Burning Oath","count":1250,"rarity":"epic"},
  },
  interactionPrompt: {
    default: {"entity":0,"at":"2 0 1","height":1.2,"action":"interact","verb":"Open","name":"Chest","disabled":false,"reason":""},
    disabled: {"entity":0,"at":"4 0 -0.5","height":1.2,"action":"interact","verb":"Open","name":"Vault","disabled":true,"reason":"Locked"},
    overflow: {"entity":0,"at":"1 0 0","height":1.2,"action":"interact","verb":"Pick up and equip","name":"Ember Blade of the Ninefold Gate","disabled":false,"reason":""},
  },
  waypoint: {
    default: {"entity":0,"at":"0 0 -8","height":1.5,"kind":"objective","icon":"","label":"Defend","distance":142,"units":"m"},
    ping: {"entity":0,"at":"-8 0 -6","height":1.5,"kind":"ping","icon":"","label":"","distance":38,"units":"m"},
    danger: {"entity":0,"at":"8 0 -7","height":1.5,"kind":"danger","icon":"","label":"Enemy spotted","distance":61,"units":"m"},
    ally: {"entity":0,"at":"-3 0 -10","height":1.5,"kind":"ally","icon":"","label":"Mira","distance":-1,"units":"m"},
    loot: {"entity":0,"at":"4 0 -9","height":1.5,"kind":"loot","icon":"","label":"","distance":12,"units":"m"},
    offscreen: {"entity":0,"at":"60 0 -4","height":1.5,"kind":"objective","icon":"","label":"Lighthouse","distance":1240,"units":"m"},
    overflow: {"entity":0,"at":"-6 0 -9","height":1.5,"kind":"objective","icon":"","label":"Light every lamp of the northern reach","distance":12400,"units":"km"},
  },
  speechBubble: {
    default: {"entity":0,"at":"3 0 -2","height":2.2,"speaker":"Mira","text":"The bridge is out. Take the north pass.","shout":false},
    shout: {"entity":0,"at":"-4 0 -1","height":2.2,"speaker":"Grimtooth","text":"None shall pass!","shout":true},
    overflow: {"entity":0,"at":"-2 0 -3","height":2.2,"speaker":"Keeper of the Northern Lighthouse","text":"The lamps went dark three nights ago, and since then the ships have stopped coming; cross the gate before the tide turns.","shout":false},
  },
  selectionRing: {
    default: {"entity":0,"at":"3 0 -2","radius":1,"reaction":"","hover":false,"segmented":false},
    hostile: {"entity":0,"at":"-4 0 -1","radius":1.3,"reaction":"hostile","hover":false,"segmented":false},
    neutral: {"entity":0,"at":"7 0 -4","radius":1,"reaction":"neutral","hover":false,"segmented":false},
    hover: {"entity":0,"at":"0 0 2","radius":1,"reaction":"","hover":true,"segmented":false},
    segmented: {"entity":0,"at":"-2 0 -3","radius":1.5,"reaction":"hostile","hover":false,"segmented":true},
  },
  telegraph: {
    default: {"entity":0,"at":"-4 0 -1","shape":"cone","radius":6,"inner":0.5,"angle":70,"width":0.2,"direction":30,"progress":0.55,"friendly":false},
    ring: {"entity":0,"at":"-2 0 -3","shape":"ring","radius":4,"inner":0.5,"angle":70,"width":0.2,"direction":30,"progress":0.8,"friendly":false},
    disc: {"entity":0,"at":"3 0 -2","shape":"disc","radius":2.5,"inner":0.5,"angle":70,"width":0.2,"direction":30,"progress":0.3,"friendly":true},
    line: {"entity":0,"at":"6 0 0","shape":"line","radius":7,"inner":0.5,"angle":70,"width":0.2,"direction":250,"progress":0.95,"friendly":false},
  },
  pathLine: {
    default: {"at":"1 0 -1","size":8,"path":"M40 160 L80 120 L120 120 L150 70","reach":"M150 70 L170 40","endX":170,"endY":40,"label":"12 m · 2 AP","invalid":false},
    invalid: {"at":"3 0 0","size":8,"path":"M40 160 L60 120 L110 110 L130 60","reach":"","endX":130,"endY":60,"label":"Out of reach · 6 AP","invalid":true},
  },
  placementGhost: {
    default: {"at":"-7 0 0","width":3,"depth":3,"rotation":0,"cells":3,"valid":false,"label":"Blocked","range":0},
    valid: {"at":"7 0 -1","width":2,"depth":2,"rotation":0,"cells":2,"valid":true,"label":"150","range":3},
  },
  damageNumbers: {
    default: {"hits":[{"id":"1","entity":0,"at":"-4 2.4 -1","amount":128,"color":""},{"id":"2","entity":0,"at":"-3.5 2.6 -1","amount":1240,"color":"#ffc93c"}]},
    crowd: {"hits":[{"id":"0","entity":0,"at":"-3.0 2.2 -3","amount":12,"color":""},{"id":"1","entity":0,"at":"-2.6 2.2 -3","amount":128,"color":"#ffc93c"},{"id":"2","entity":0,"at":"-2.2 2.2 -3","amount":1240,"color":""},{"id":"3","entity":0,"at":"-1.7999999999999998 2.2 -3","amount":98765,"color":"#e5484d"},{"id":"4","entity":0,"at":"-1.4 2.2 -3","amount":3,"color":""},{"id":"5","entity":0,"at":"-3.0 2.6 -3","amount":4410,"color":"#4ec9ff"},{"id":"6","entity":0,"at":"-2.6 2.6 -3","amount":77,"color":""},{"id":"7","entity":0,"at":"-2.2 2.6 -3","amount":250000,"color":"#ffc93c"},{"id":"8","entity":0,"at":"-1.7999999999999998 2.6 -3","amount":9,"color":""},{"id":"9","entity":0,"at":"-1.4 2.6 -3","amount":640,"color":""}]},
  },
  overheadBars: {
    default: {"height":2,"bars":[{"id":"a","entity":0,"at":"-6 0 -3","value":72,"max":100,"color":""},{"id":"b","entity":0,"at":"6 0 -2","value":30,"max":100,"color":"#e5484d"}]},
    crowd: {"height":1.6,"bars":[{"id":"u0","entity":0,"at":"-9.0 0 -6.0","value":0,"max":100,"color":""},{"id":"u1","entity":0,"at":"-6.6 0 -6.0","value":37,"max":100,"color":"#e5484d"},{"id":"u2","entity":0,"at":"-4.2 0 -6.0","value":74,"max":100,"color":""},{"id":"u3","entity":0,"at":"-1.8000000000000007 0 -6.0","value":10,"max":100,"color":"#4ec9ff"},{"id":"u4","entity":0,"at":"0.5999999999999996 0 -6.0","value":47,"max":100,"color":""},{"id":"u5","entity":0,"at":"3.0 0 -6.0","value":84,"max":100,"color":"#e5484d"},{"id":"u6","entity":0,"at":"5.399999999999999 0 -6.0","value":20,"max":100,"color":""},{"id":"u7","entity":0,"at":"7.800000000000001 0 -6.0","value":57,"max":100,"color":"#4ec9ff"},{"id":"u8","entity":0,"at":"-9.0 0 -4.8","value":94,"max":100,"color":""},{"id":"u9","entity":0,"at":"-6.6 0 -4.8","value":30,"max":100,"color":"#e5484d"},{"id":"u10","entity":0,"at":"-4.2 0 -4.8","value":67,"max":100,"color":""},{"id":"u11","entity":0,"at":"-1.8000000000000007 0 -4.8","value":3,"max":100,"color":"#4ec9ff"},{"id":"u12","entity":0,"at":"0.5999999999999996 0 -4.8","value":40,"max":100,"color":""},{"id":"u13","entity":0,"at":"3.0 0 -4.8","value":77,"max":100,"color":"#e5484d"},{"id":"u14","entity":0,"at":"5.399999999999999 0 -4.8","value":13,"max":100,"color":""},{"id":"u15","entity":0,"at":"7.800000000000001 0 -4.8","value":50,"max":100,"color":"#4ec9ff"},{"id":"u16","entity":0,"at":"-9.0 0 -3.6","value":87,"max":100,"color":""},{"id":"u17","entity":0,"at":"-6.6 0 -3.6","value":23,"max":100,"color":"#e5484d"},{"id":"u18","entity":0,"at":"-4.2 0 -3.6","value":60,"max":100,"color":""},{"id":"u19","entity":0,"at":"-1.8000000000000007 0 -3.6","value":97,"max":100,"color":"#4ec9ff"},{"id":"u20","entity":0,"at":"0.5999999999999996 0 -3.6","value":33,"max":100,"color":""},{"id":"u21","entity":0,"at":"3.0 0 -3.6","value":70,"max":100,"color":"#e5484d"},{"id":"u22","entity":0,"at":"5.399999999999999 0 -3.6","value":6,"max":100,"color":""},{"id":"u23","entity":0,"at":"7.800000000000001 0 -3.6","value":43,"max":100,"color":"#4ec9ff"}]},
  },
};

export const NAMES: Record<string, string> = {"nameplate":"Nameplate","lootLabel":"LootLabel","interactionPrompt":"InteractionPrompt","waypoint":"Waypoint","speechBubble":"SpeechBubble","selectionRing":"SelectionRing","telegraph":"Telegraph","pathLine":"PathLine","placementGhost":"PlacementGhost","damageNumbers":"DamageNumbers","overheadBars":"OverheadBars"};
