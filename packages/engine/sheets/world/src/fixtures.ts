// The world sheet's data (docs/ui.md §3.1): every world label's fixture
// states (its document's fixtures merged over its default, kept equal to
// the documents by hand; generated from them), and where each page puts
// them, in the pixels of the mockup frame it was drawn for
// (world-desktop and world-topdown at 1920 × 1080, world-phone at 390 × 844
// and 844 × 390). src/main.ts turns a point into a world point and a
// ground mark's width (w, px) into world units.

export const STATES: Record<string, Record<string, Record<string, unknown>>> = {
 "Nameplate": {
  "default": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.1,
   "name": "Vorric the Hollow",
   "title": "Ashbound Ritualist",
   "level": "43",
   "reaction": "hostile",
   "health": 0.91,
   "shield": 0.18,
   "castName": "Shadow Volley",
   "castProgress": 0.64,
   "castTime": "1.2",
   "castState": "",
   "icon": "",
   "elite": true,
   "boss": false,
   "targeted": false,
   "dead": false,
   "lootable": false,
   "compact": false,
   "maxDistance": 80
  },
  "friendly": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.1,
   "name": "Mira Ashgrove",
   "title": "Warden’s Scout",
   "level": "",
   "reaction": "friendly",
   "health": -1,
   "shield": 0,
   "castName": "",
   "castProgress": 0,
   "castTime": "",
   "castState": "",
   "icon": "",
   "elite": false,
   "boss": false,
   "targeted": false,
   "dead": false,
   "lootable": false,
   "compact": false,
   "maxDistance": 80
  },
  "neutral": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.1,
   "name": "Old Tam",
   "title": "Ferryman",
   "level": "12",
   "reaction": "",
   "health": 1,
   "shield": 0,
   "castName": "",
   "castProgress": 0,
   "castTime": "",
   "castState": "",
   "icon": "",
   "elite": false,
   "boss": false,
   "targeted": false,
   "dead": false,
   "lootable": false,
   "compact": false,
   "maxDistance": 80
  },
  "hostile": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.1,
   "name": "Ashbound Thrall",
   "title": "",
   "level": "41",
   "reaction": "hostile",
   "health": 0.78,
   "shield": 0,
   "castName": "",
   "castProgress": 0,
   "castTime": "",
   "castState": "",
   "icon": "",
   "elite": false,
   "boss": false,
   "targeted": false,
   "dead": false,
   "lootable": false,
   "compact": false,
   "maxDistance": 80
  },
  "player": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.1,
   "name": "Sylvenne",
   "title": "Ninefold Wardens",
   "level": "42",
   "reaction": "player",
   "health": 0.86,
   "shield": 0.1,
   "castName": "",
   "castProgress": 0,
   "castTime": "",
   "castState": "",
   "icon": "",
   "elite": false,
   "boss": false,
   "targeted": false,
   "dead": false,
   "lootable": false,
   "compact": false,
   "maxDistance": 80
  },
  "boss": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.1,
   "name": "Ashen Colossus",
   "title": "Warden of the Ninefold Gate",
   "level": "",
   "reaction": "hostile",
   "health": 0.58,
   "shield": 0,
   "castName": "",
   "castProgress": 0,
   "castTime": "",
   "castState": "",
   "icon": "",
   "elite": false,
   "boss": true,
   "targeted": false,
   "dead": false,
   "lootable": false,
   "compact": false,
   "maxDistance": 80
  },
  "targeted": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.1,
   "name": "Ashbound Thrall",
   "title": "",
   "level": "41",
   "reaction": "hostile",
   "health": 0.78,
   "shield": 0,
   "castName": "",
   "castProgress": 0,
   "castTime": "",
   "castState": "",
   "icon": "",
   "elite": false,
   "boss": false,
   "targeted": true,
   "dead": false,
   "lootable": false,
   "compact": false,
   "maxDistance": 80
  },
  "low": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.1,
   "name": "Ashbound Thrall",
   "title": "",
   "level": "41",
   "reaction": "hostile",
   "health": 0.22,
   "shield": 0,
   "castName": "",
   "castProgress": 0,
   "castTime": "",
   "castState": "",
   "icon": "",
   "elite": false,
   "boss": false,
   "targeted": false,
   "dead": false,
   "lootable": false,
   "compact": false,
   "maxDistance": 80
  },
  "casting": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.1,
   "name": "Vorric the Hollow",
   "title": "Ashbound Ritualist",
   "level": "43",
   "reaction": "hostile",
   "health": 0.91,
   "shield": 0.18,
   "castName": "Shadow Volley",
   "castProgress": 0.64,
   "castTime": "1.2",
   "castState": "",
   "icon": "",
   "elite": true,
   "boss": false,
   "targeted": false,
   "dead": false,
   "lootable": false,
   "compact": false,
   "maxDistance": 80
  },
  "locked": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.1,
   "name": "Ashen Colossus",
   "title": "Warden of the Ninefold Gate",
   "level": "",
   "reaction": "hostile",
   "health": 0.58,
   "shield": 0,
   "castName": "Cataclysm",
   "castProgress": 0.62,
   "castTime": "2.4",
   "castState": "locked",
   "icon": "",
   "elite": false,
   "boss": true,
   "targeted": true,
   "dead": false,
   "lootable": false,
   "compact": false,
   "maxDistance": 80
  },
  "interrupted": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.1,
   "name": "Vorric the Hollow",
   "title": "Ashbound Ritualist",
   "level": "43",
   "reaction": "hostile",
   "health": 0.91,
   "shield": 0.18,
   "castName": "Shadow Volley",
   "castProgress": 1,
   "castTime": "",
   "castState": "interrupted",
   "icon": "",
   "elite": true,
   "boss": false,
   "targeted": false,
   "dead": false,
   "lootable": false,
   "compact": false,
   "maxDistance": 80
  },
  "dead": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.1,
   "name": "Ashbound Thrall",
   "title": "",
   "level": "41",
   "reaction": "hostile",
   "health": 0,
   "shield": 0,
   "castName": "",
   "castProgress": 0,
   "castTime": "",
   "castState": "",
   "icon": "",
   "elite": false,
   "boss": false,
   "targeted": false,
   "dead": true,
   "lootable": true,
   "compact": false,
   "maxDistance": 80
  },
  "compact": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.1,
   "name": "Ashbound Thrall",
   "title": "",
   "level": "41",
   "reaction": "hostile",
   "health": 0.62,
   "shield": 0,
   "castName": "",
   "castProgress": 0,
   "castTime": "",
   "castState": "",
   "icon": "",
   "elite": false,
   "boss": false,
   "targeted": false,
   "dead": false,
   "lootable": false,
   "compact": true,
   "maxDistance": 80
  },
  "overflow": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.1,
   "name": "Ysolde the Unbroken, Warden of the Ninefold Gate",
   "title": "Guardian of the Eastern Marches and the Ninefold Gate",
   "level": "120",
   "reaction": "hostile",
   "health": 0.12,
   "shield": 0.15,
   "castName": "Ritual of the Ninefold Gate Unending",
   "castProgress": 0.85,
   "castTime": "12.4",
   "castState": "",
   "icon": "",
   "elite": false,
   "boss": true,
   "targeted": true,
   "dead": false,
   "lootable": false,
   "compact": false,
   "maxDistance": 80
  }
 },
 "LootLabel": {
  "default": {
   "entity": 0,
   "at": "0 0 0",
   "height": 0,
   "name": "Ember Blade",
   "count": 1,
   "rarity": "epic",
   "hovered": false,
   "action": "",
   "more": 0,
   "tiers": []
  },
  "common": {
   "entity": 0,
   "at": "0 0 0",
   "height": 0,
   "name": "Rusted Buckle",
   "count": 1,
   "rarity": "",
   "hovered": false,
   "action": "",
   "more": 0,
   "tiers": []
  },
  "stack": {
   "entity": 0,
   "at": "0 0 0",
   "height": 0,
   "name": "Gold",
   "count": 214,
   "rarity": "",
   "hovered": false,
   "action": "",
   "more": 0,
   "tiers": []
  },
  "uncommon": {
   "entity": 0,
   "at": "0 0 0",
   "height": 0,
   "name": "Wolfhide Bracers",
   "count": 1,
   "rarity": "uncommon",
   "hovered": false,
   "action": "",
   "more": 0,
   "tiers": []
  },
  "rare": {
   "entity": 0,
   "at": "0 0 0",
   "height": 0,
   "name": "Stormglass Pendant",
   "count": 1,
   "rarity": "rare",
   "hovered": false,
   "action": "",
   "more": 0,
   "tiers": []
  },
  "legendary": {
   "entity": 0,
   "at": "0 0 0",
   "height": 0,
   "name": "Crown of the Ninefold King",
   "count": 1,
   "rarity": "legendary",
   "hovered": false,
   "action": "",
   "more": 0,
   "tiers": []
  },
  "hovered": {
   "entity": 0,
   "at": "0 0 0",
   "height": 0,
   "name": "Ember Blade",
   "count": 1,
   "rarity": "epic",
   "hovered": true,
   "action": "pick",
   "more": 0,
   "tiers": []
  },
  "more": {
   "entity": 0,
   "at": "0 0 0",
   "height": 0,
   "name": "",
   "count": 1,
   "rarity": "",
   "hovered": false,
   "action": "",
   "more": 6,
   "tiers": [
    {
     "id": "a",
     "rarity": "rare"
    },
    {
     "id": "b",
     "rarity": "uncommon"
    },
    {
     "id": "c",
     "rarity": "uncommon"
    },
    {
     "id": "d",
     "rarity": ""
    }
   ]
  },
  "overflow": {
   "entity": 0,
   "at": "0 0 0",
   "height": 0,
   "name": "Blade of the Ninefold King’s Last Oath, Reforged in Ember",
   "count": 1250,
   "rarity": "legendary",
   "hovered": false,
   "action": "",
   "more": 0,
   "tiers": []
  }
 },
 "InteractionPrompt": {
  "default": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.2,
   "action": "",
   "key": "E",
   "pad": false,
   "verb": "Open",
   "name": "Gilded Chest",
   "pressed": false,
   "hold": -1,
   "disabled": false,
   "reason": "",
   "maxDistance": 6
  },
  "pressed": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.2,
   "action": "",
   "key": "E",
   "pad": false,
   "verb": "Open",
   "name": "Gilded Chest",
   "pressed": true,
   "hold": -1,
   "disabled": false,
   "reason": "",
   "maxDistance": 6
  },
  "hold": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.2,
   "action": "",
   "key": "F",
   "pad": false,
   "verb": "Hold · Revive",
   "name": "Brannoch",
   "pressed": true,
   "hold": 0.45,
   "disabled": false,
   "reason": "",
   "maxDistance": 6
  },
  "done": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.2,
   "action": "",
   "key": "F",
   "pad": false,
   "verb": "Revive",
   "name": "Brannoch",
   "pressed": false,
   "hold": 1,
   "disabled": false,
   "reason": "",
   "maxDistance": 6
  },
  "disabled": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.2,
   "action": "",
   "key": "E",
   "pad": false,
   "verb": "Open",
   "name": "Iron Door",
   "pressed": false,
   "hold": -1,
   "disabled": true,
   "reason": "Needs the Warden’s key",
   "maxDistance": 6
  },
  "gamepad": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.2,
   "action": "",
   "key": "A",
   "pad": true,
   "verb": "Talk",
   "name": "Mira Ashgrove",
   "pressed": false,
   "hold": -1,
   "disabled": false,
   "reason": "",
   "maxDistance": 6
  },
  "action": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.2,
   "action": "interact",
   "key": "",
   "pad": false,
   "verb": "Open",
   "name": "Gilded Chest",
   "pressed": false,
   "hold": -1,
   "disabled": false,
   "reason": "",
   "maxDistance": 6
  },
  "overflow": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.2,
   "action": "",
   "key": "Left Shoulder",
   "pad": false,
   "verb": "Pick up and equip",
   "name": "Blade of the Ninefold King’s Last Oath, Reforged in Ember",
   "pressed": false,
   "hold": -1,
   "disabled": false,
   "reason": "",
   "maxDistance": 6
  }
 },
 "SpeechBubble": {
  "default": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.2,
   "speaker": "Mira",
   "reaction": "",
   "text": "The Colossus guards the gate. Break its shield, then strike the runes!",
   "shout": false,
   "left": 0.64
  },
  "short": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.2,
   "speaker": "Mira",
   "reaction": "",
   "text": "Over here!",
   "shout": false,
   "left": 0.3
  },
  "shout": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.2,
   "speaker": "Vorric",
   "reaction": "hostile",
   "text": "Kneel before the ash!",
   "shout": true,
   "left": 0.8
  },
  "nameless": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.2,
   "speaker": "",
   "reaction": "",
   "text": "Wait for me at the bridge.",
   "shout": false,
   "left": -1
  },
  "overflow": {
   "entity": 0,
   "at": "0 0 0",
   "height": 2.2,
   "speaker": "Keeper of the Northern Lighthouse",
   "reaction": "",
   "text": "The lamps went dark three nights ago, and since then the ships have stopped coming; cross the gate before the tide turns.",
   "shout": false,
   "left": 0.12
  }
 },
 "Telegraph": {
  "default": {
   "entity": 0,
   "at": "0 0 0",
   "shape": "",
   "radius": 4,
   "inner": 0.45,
   "angle": 70,
   "width": 0.2,
   "direction": 0,
   "progress": 0.62,
   "friendly": false,
   "hit": false
  },
  "cone": {
   "entity": 0,
   "at": "0 0 0",
   "shape": "cone",
   "radius": 6,
   "inner": 0.45,
   "angle": 70,
   "width": 0.2,
   "direction": 0,
   "progress": 0.55,
   "friendly": false,
   "hit": false
  },
  "line": {
   "entity": 0,
   "at": "0 0 0",
   "shape": "line",
   "radius": 7,
   "inner": 0.45,
   "angle": 70,
   "width": 0.2,
   "direction": 0,
   "progress": 0.8,
   "friendly": false,
   "hit": false
  },
  "ring": {
   "entity": 0,
   "at": "0 0 0",
   "shape": "ring",
   "radius": 4,
   "inner": 0.45,
   "angle": 70,
   "width": 0.2,
   "direction": 0,
   "progress": 0.4,
   "friendly": false,
   "hit": false
  },
  "impact": {
   "entity": 0,
   "at": "0 0 0",
   "shape": "",
   "radius": 4,
   "inner": 0.45,
   "angle": 70,
   "width": 0.2,
   "direction": 0,
   "progress": 1,
   "friendly": false,
   "hit": true
  },
  "ally": {
   "entity": 0,
   "at": "0 0 0",
   "shape": "",
   "radius": 4,
   "inner": 0.45,
   "angle": 70,
   "width": 0.2,
   "direction": 0,
   "progress": 0.5,
   "friendly": true,
   "hit": false
  }
 },
 "SelectionRing": {
  "default": {
   "entity": 0,
   "at": "0 0 0",
   "radius": 1,
   "side": "",
   "leader": false,
   "target": false,
   "hover": false
  },
  "leader": {
   "entity": 0,
   "at": "0 0 0",
   "radius": 1,
   "side": "",
   "leader": true,
   "target": false,
   "hover": false
  },
  "ally": {
   "entity": 0,
   "at": "0 0 0",
   "radius": 1,
   "side": "ally",
   "leader": false,
   "target": false,
   "hover": false
  },
  "target": {
   "entity": 0,
   "at": "0 0 0",
   "radius": 1.2,
   "side": "enemy",
   "leader": false,
   "target": true,
   "hover": false
  },
  "neutral": {
   "entity": 0,
   "at": "0 0 0",
   "radius": 1,
   "side": "neutral",
   "leader": false,
   "target": false,
   "hover": false
  },
  "hover": {
   "entity": 0,
   "at": "0 0 0",
   "radius": 1,
   "side": "enemy",
   "leader": false,
   "target": false,
   "hover": true
  }
 },
 "PlacementGhost": {
  "default": {
   "at": "0 0 0",
   "width": 3,
   "depth": 3,
   "rotation": 45,
   "columns": 3,
   "cells": [
    {
     "id": "0",
     "blocked": false
    },
    {
     "id": "1",
     "blocked": false
    },
    {
     "id": "2",
     "blocked": false
    },
    {
     "id": "3",
     "blocked": false
    },
    {
     "id": "4",
     "blocked": false
    },
    {
     "id": "5",
     "blocked": false
    },
    {
     "id": "6",
     "blocked": false
    },
    {
     "id": "7",
     "blocked": false
    },
    {
     "id": "8",
     "blocked": false
    }
   ],
   "valid": true,
   "label": "Watchtower · 120",
   "range": 2.8
  },
  "blocked": {
   "at": "0 0 0",
   "width": 3,
   "depth": 3,
   "rotation": 45,
   "columns": 3,
   "cells": [
    {
     "id": "0",
     "blocked": true
    },
    {
     "id": "1",
     "blocked": true
    },
    {
     "id": "2",
     "blocked": true
    },
    {
     "id": "3",
     "blocked": true
    },
    {
     "id": "4",
     "blocked": true
    },
    {
     "id": "5",
     "blocked": true
    },
    {
     "id": "6",
     "blocked": true
    },
    {
     "id": "7",
     "blocked": true
    },
    {
     "id": "8",
     "blocked": true
    }
   ],
   "valid": false,
   "label": "Blocked · water",
   "range": 0
  },
  "partial": {
   "at": "0 0 0",
   "width": 3,
   "depth": 3,
   "rotation": 45,
   "columns": 3,
   "cells": [
    {
     "id": "0",
     "blocked": true
    },
    {
     "id": "1",
     "blocked": true
    },
    {
     "id": "2",
     "blocked": false
    },
    {
     "id": "3",
     "blocked": true
    },
    {
     "id": "4",
     "blocked": false
    },
    {
     "id": "5",
     "blocked": false
    },
    {
     "id": "6",
     "blocked": false
    },
    {
     "id": "7",
     "blocked": false
    },
    {
     "id": "8",
     "blocked": false
    }
   ],
   "valid": false,
   "label": "3 cells on rock",
   "range": 0
  }
 },
 "Waypoint": {
  "default": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.5,
   "kind": "",
   "label": "The Ninefold Gate",
   "distance": "412",
   "units": "m",
   "range": "",
   "edge": ""
  },
  "near": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.5,
   "kind": "",
   "label": "Ninefold Gate",
   "distance": "12",
   "units": "m",
   "range": "near",
   "edge": ""
  },
  "far": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.5,
   "kind": "",
   "label": "The Ninefold Gate",
   "distance": "1,240",
   "units": "m",
   "range": "far",
   "edge": ""
  },
  "ping": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.5,
   "kind": "ping",
   "label": "Look here",
   "distance": "34",
   "units": "m",
   "range": "",
   "edge": ""
  },
  "danger": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.5,
   "kind": "danger",
   "label": "Enemy",
   "distance": "58",
   "units": "m",
   "range": "",
   "edge": ""
  },
  "ally": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.5,
   "kind": "ally",
   "label": "Sylvenne",
   "distance": "22",
   "units": "m",
   "range": "",
   "edge": ""
  },
  "loot": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.5,
   "kind": "loot",
   "label": "Cache",
   "distance": "140",
   "units": "m",
   "range": "",
   "edge": ""
  },
  "offscreen": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.5,
   "kind": "ally",
   "label": "Brannoch",
   "distance": "86",
   "units": "m",
   "range": "",
   "edge": "left"
  },
  "offscreenRight": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.5,
   "kind": "danger",
   "label": "Enemy",
   "distance": "58",
   "units": "m",
   "range": "",
   "edge": "right"
  },
  "overflow": {
   "entity": 0,
   "at": "0 0 0",
   "height": 1.5,
   "kind": "",
   "label": "Light every lamp of the northern reach before nightfall",
   "distance": "12,400",
   "units": "km",
   "range": "",
   "edge": ""
  }
 },
 "PathLine": {
  "default": {
   "at": "0 0 0",
   "size": 10,
   "path": "M30 150 L80 120 L120 110 L150 70",
   "reach": "M150 70 L172 40",
   "stops": [
    {
     "id": "1",
     "x": 80,
     "y": 120
    },
    {
     "id": "2",
     "x": 120,
     "y": 110
    }
   ],
   "endX": 172,
   "endY": 40,
   "label": "64 m · 3 orders",
   "invalid": false
  },
  "invalid": {
   "at": "0 0 0",
   "size": 10,
   "path": "M30 150 L80 120 L120 110 L150 70",
   "reach": "",
   "stops": [],
   "endX": 150,
   "endY": 70,
   "label": "No path",
   "invalid": true
  }
 }
};

export type Placed = { kind: string; state: string; x: number; y: number; w?: number; id?: string; over?: Record<string, unknown> };

/** Each page's layout by breakpoint: the frame's width and height and what stands where. A page without a phone layout shows the middle of its desktop layout, as its art covers the screen. */
export const LAYOUTS: Record<string, Record<string, [number, number, Placed[]]>> = {
 "scene": {
  "desktop": [
   1920,
   1080,
   [
    {
     "kind": "Telegraph",
     "state": "default",
     "x": 1236,
     "y": 646,
     "w": 470
    },
    {
     "kind": "OverheadBars",
     "state": "ally",
     "x": 118,
     "y": 636,
     "over": {
      "value": 90,
      "resource": 70,
      "resourceMax": 100
     }
    },
    {
     "kind": "OverheadBars",
     "state": "ally",
     "x": 204,
     "y": 654,
     "id": "bar2",
     "over": {
      "value": 24,
      "resource": 35,
      "resourceMax": 100
     }
    },
    {
     "kind": "Nameplate",
     "state": "friendly",
     "x": 398,
     "y": 560
    },
    {
     "kind": "SpeechBubble",
     "state": "default",
     "x": 398,
     "y": 500
    },
    {
     "kind": "Nameplate",
     "state": "hostile",
     "x": 1528,
     "y": 612
    },
    {
     "kind": "Nameplate",
     "state": "default",
     "x": 1662,
     "y": 575
    },
    {
     "kind": "Nameplate",
     "state": "low",
     "x": 1790,
     "y": 622
    },
    {
     "kind": "Nameplate",
     "state": "locked",
     "x": 1236,
     "y": 150
    },
    {
     "kind": "InteractionPrompt",
     "state": "default",
     "x": 1272,
     "y": 766
    },
    {
     "kind": "LootLabel",
     "state": "rare",
     "x": 1420,
     "y": 968
    },
    {
     "kind": "LootLabel",
     "state": "uncommon",
     "x": 1606,
     "y": 892
    },
    {
     "kind": "LootLabel",
     "state": "common",
     "x": 1790,
     "y": 880
    },
    {
     "kind": "LootLabel",
     "state": "hovered",
     "x": 1500,
     "y": 1052
    },
    {
     "kind": "LootLabel",
     "state": "legendary",
     "x": 1626,
     "y": 990
    },
    {
     "kind": "LootLabel",
     "state": "rare",
     "x": 1730,
     "y": 960,
     "id": "pile0",
     "over": {
      "name": "Vial of Embers"
     }
    },
    {
     "kind": "LootLabel",
     "state": "stack",
     "x": 1730,
     "y": 1004,
     "id": "pile1",
     "over": {}
    },
    {
     "kind": "LootLabel",
     "state": "more",
     "x": 1730,
     "y": 1048,
     "id": "pilemore",
     "over": {
      "more": 6
     }
    },
    {
     "kind": "Waypoint",
     "state": "default",
     "x": 862,
     "y": 168
    },
    {
     "kind": "Waypoint",
     "state": "offscreen",
     "x": -160,
     "y": 330
    }
   ]
  ],
  "portrait": [
   390,
   844,
   [
    {
     "kind": "Telegraph",
     "state": "default",
     "x": 362,
     "y": 512,
     "w": 250
    },
    {
     "kind": "Nameplate",
     "state": "locked",
     "x": 230,
     "y": 150
    },
    {
     "kind": "Waypoint",
     "state": "default",
     "x": 62,
     "y": 330,
     "over": {
      "label": ""
     }
    },
    {
     "kind": "Waypoint",
     "state": "offscreen",
     "x": -120,
     "y": 410
    },
    {
     "kind": "SpeechBubble",
     "state": "default",
     "x": 170,
     "y": 318,
     "over": {
      "text": "Break its shield, then strike the runes!"
     }
    },
    {
     "kind": "Nameplate",
     "state": "compact",
     "x": 112,
     "y": 520
    },
    {
     "kind": "Nameplate",
     "state": "default",
     "x": 250,
     "y": 450,
     "over": {
      "title": ""
     }
    },
    {
     "kind": "Nameplate",
     "state": "compact",
     "x": 290,
     "y": 530,
     "id": "compact2",
     "over": {
      "health": 0.22
     }
    },
    {
     "kind": "InteractionPrompt",
     "state": "default",
     "x": 260,
     "y": 660
    },
    {
     "kind": "LootLabel",
     "state": "legendary",
     "x": 230,
     "y": 760
    },
    {
     "kind": "LootLabel",
     "state": "more",
     "x": 300,
     "y": 800,
     "over": {
      "more": 7
     }
    }
   ]
  ],
  "landscape": [
   844,
   390,
   [
    {
     "kind": "Telegraph",
     "state": "default",
     "x": 544,
     "y": 224,
     "w": 207
    },
    {
     "kind": "OverheadBars",
     "state": "ally",
     "x": 70,
     "y": 222,
     "over": {
      "value": 90,
      "resource": 70,
      "resourceMax": 100
     }
    },
    {
     "kind": "OverheadBars",
     "state": "ally",
     "x": 104,
     "y": 232,
     "id": "bar2",
     "over": {
      "value": 24,
      "resource": 35,
      "resourceMax": 100
     }
    },
    {
     "kind": "Nameplate",
     "state": "friendly",
     "x": 175,
     "y": 200
    },
    {
     "kind": "SpeechBubble",
     "state": "default",
     "x": 152,
     "y": 148,
     "over": {
      "text": "Break its shield, then strike the runes!"
     }
    },
    {
     "kind": "Nameplate",
     "state": "compact",
     "x": 650,
     "y": 214
    },
    {
     "kind": "Nameplate",
     "state": "default",
     "x": 715,
     "y": 180,
     "over": {
      "title": ""
     }
    },
    {
     "kind": "Nameplate",
     "state": "compact",
     "x": 770,
     "y": 216,
     "id": "compact2",
     "over": {
      "health": 0.22
     }
    },
    {
     "kind": "Nameplate",
     "state": "locked",
     "x": 470,
     "y": 112
    },
    {
     "kind": "Waypoint",
     "state": "default",
     "x": 320,
     "y": 70,
     "over": {
      "label": ""
     }
    },
    {
     "kind": "Waypoint",
     "state": "offscreen",
     "x": -120,
     "y": 290
    },
    {
     "kind": "InteractionPrompt",
     "state": "default",
     "x": 540,
     "y": 290
    },
    {
     "kind": "LootLabel",
     "state": "default",
     "x": 560,
     "y": 368
    },
    {
     "kind": "LootLabel",
     "state": "more",
     "x": 720,
     "y": 368,
     "over": {
      "more": 6
     }
    }
   ]
  ]
 },
 "topdown": {
  "desktop": [
   1920,
   1080,
   [
    {
     "kind": "PlacementGhost",
     "state": "default",
     "x": 562,
     "y": 542,
     "w": 118
    },
    {
     "kind": "PlacementGhost",
     "state": "partial",
     "x": 1060,
     "y": 330,
     "w": 120,
     "over": {
      "label": "Blocked · water"
     }
    },
    {
     "kind": "SelectionRing",
     "state": "default",
     "x": 1083,
     "y": 577,
     "w": 44,
     "id": "own0"
    },
    {
     "kind": "SelectionRing",
     "state": "default",
     "x": 1133,
     "y": 557,
     "w": 44,
     "id": "own1"
    },
    {
     "kind": "SelectionRing",
     "state": "default",
     "x": 1163,
     "y": 579,
     "w": 44,
     "id": "own2"
    },
    {
     "kind": "SelectionRing",
     "state": "default",
     "x": 1193,
     "y": 543,
     "w": 44,
     "id": "own3"
    },
    {
     "kind": "SelectionRing",
     "state": "default",
     "x": 1220,
     "y": 567,
     "w": 44,
     "id": "own4"
    },
    {
     "kind": "SelectionRing",
     "state": "leader",
     "x": 1280,
     "y": 563,
     "w": 44,
     "id": "own5"
    },
    {
     "kind": "SelectionRing",
     "state": "ally",
     "x": 827,
     "y": 627,
     "w": 40,
     "id": "ally0",
     "over": {
      "side": "ally",
      "hover": false
     }
    },
    {
     "kind": "SelectionRing",
     "state": "ally",
     "x": 893,
     "y": 597,
     "w": 40,
     "id": "ally1",
     "over": {
      "side": "ally",
      "hover": false
     }
    },
    {
     "kind": "SelectionRing",
     "state": "ally",
     "x": 927,
     "y": 630,
     "w": 40,
     "id": "ally2",
     "over": {
      "side": "ally",
      "hover": false
     }
    },
    {
     "kind": "SelectionRing",
     "state": "hover",
     "x": 967,
     "y": 577,
     "w": 40,
     "id": "ally3",
     "over": {
      "side": "ally",
      "hover": true
     }
    },
    {
     "kind": "SelectionRing",
     "state": "ally",
     "x": 1020,
     "y": 613,
     "w": 40,
     "id": "ally4",
     "over": {
      "side": "ally",
      "hover": false
     }
    },
    {
     "kind": "SelectionRing",
     "state": "ally",
     "x": 1027,
     "y": 560,
     "w": 40,
     "id": "ally5",
     "over": {
      "side": "ally",
      "hover": false
     }
    },
    {
     "kind": "SelectionRing",
     "state": "target",
     "x": 1600,
     "y": 452,
     "w": 50
    },
    {
     "kind": "SelectionRing",
     "state": "hover",
     "x": 1640,
     "y": 478,
     "w": 40,
     "id": "foehover"
    },
    {
     "kind": "PathLine",
     "state": "default",
     "x": 1420,
     "y": 500,
     "w": 260
    },
    {
     "kind": "OverheadBars",
     "state": "own",
     "x": 1083,
     "y": 531,
     "id": "bo0",
     "over": {
      "value": 100
     }
    },
    {
     "kind": "OverheadBars",
     "state": "own",
     "x": 1133,
     "y": 511,
     "id": "bo1",
     "over": {
      "value": 90
     }
    },
    {
     "kind": "OverheadBars",
     "state": "own",
     "x": 1163,
     "y": 533,
     "id": "bo2",
     "over": {
      "value": 62
     }
    },
    {
     "kind": "OverheadBars",
     "state": "own",
     "x": 1193,
     "y": 497,
     "id": "bo3",
     "over": {
      "value": 100
     }
    },
    {
     "kind": "OverheadBars",
     "state": "own",
     "x": 1220,
     "y": 521,
     "id": "bo4",
     "over": {
      "value": 80
     }
    },
    {
     "kind": "OverheadBars",
     "state": "own",
     "x": 1280,
     "y": 517,
     "id": "bo5",
     "over": {
      "value": 95
     }
    },
    {
     "kind": "OverheadBars",
     "state": "ally",
     "x": 827,
     "y": 577,
     "id": "ba0",
     "over": {
      "value": 100
     }
    },
    {
     "kind": "OverheadBars",
     "state": "ally",
     "x": 893,
     "y": 547,
     "id": "ba1",
     "over": {
      "value": 70
     }
    },
    {
     "kind": "OverheadBars",
     "state": "ally",
     "x": 927,
     "y": 580,
     "id": "ba2",
     "over": {
      "value": 100
     }
    },
    {
     "kind": "OverheadBars",
     "state": "ally",
     "x": 967,
     "y": 527,
     "id": "ba3",
     "over": {
      "value": 50
     }
    },
    {
     "kind": "OverheadBars",
     "state": "ally",
     "x": 1020,
     "y": 563,
     "id": "ba4",
     "over": {
      "value": 90
     }
    },
    {
     "kind": "OverheadBars",
     "state": "ally",
     "x": 1027,
     "y": 510,
     "id": "ba5",
     "over": {
      "value": 100
     }
    },
    {
     "kind": "OverheadBars",
     "state": "enemy",
     "x": 1556,
     "y": 434,
     "id": "bf0",
     "over": {
      "value": 70
     }
    },
    {
     "kind": "OverheadBars",
     "state": "enemy",
     "x": 1600,
     "y": 412,
     "id": "bf1",
     "over": {
      "value": 100
     }
    },
    {
     "kind": "OverheadBars",
     "state": "enemy",
     "x": 1640,
     "y": 438,
     "id": "bf2",
     "over": {
      "value": 45
     }
    },
    {
     "kind": "OverheadBars",
     "state": "enemy",
     "x": 1592,
     "y": 460,
     "id": "bf3",
     "over": {
      "value": 88
     }
    },
    {
     "kind": "Nameplate",
     "state": "hostile",
     "x": 1600,
     "y": 372,
     "over": {
      "name": "Warband Raider",
      "level": "6",
      "health": 1
     }
    }
   ]
  ]
 },
 "states": {
  "desktop": [
   1920,
   1080,
   [
    {
     "kind": "Nameplate",
     "state": "friendly",
     "x": 230,
     "y": 212.85714285714286,
     "id": "Nameplate-friendly"
    },
    {
     "kind": "Nameplate",
     "state": "neutral",
     "x": 230,
     "y": 350.0,
     "id": "Nameplate-neutral"
    },
    {
     "kind": "Nameplate",
     "state": "hostile",
     "x": 230,
     "y": 487.14285714285717,
     "id": "Nameplate-hostile"
    },
    {
     "kind": "Nameplate",
     "state": "player",
     "x": 230,
     "y": 624.2857142857142,
     "id": "Nameplate-player"
    },
    {
     "kind": "Nameplate",
     "state": "boss",
     "x": 230,
     "y": 761.4285714285713,
     "id": "Nameplate-boss"
    },
    {
     "kind": "Nameplate",
     "state": "targeted",
     "x": 230,
     "y": 898.5714285714286,
     "id": "Nameplate-targeted"
    },
    {
     "kind": "Nameplate",
     "state": "low",
     "x": 230,
     "y": 1035.7142857142858,
     "id": "Nameplate-low"
    },
    {
     "kind": "Nameplate",
     "state": "casting",
     "x": 560,
     "y": 230.0,
     "id": "Nameplate-casting"
    },
    {
     "kind": "Nameplate",
     "state": "locked",
     "x": 560,
     "y": 390.0,
     "id": "Nameplate-locked"
    },
    {
     "kind": "Nameplate",
     "state": "interrupted",
     "x": 560,
     "y": 550.0,
     "id": "Nameplate-interrupted"
    },
    {
     "kind": "Nameplate",
     "state": "dead",
     "x": 560,
     "y": 710.0,
     "id": "Nameplate-dead"
    },
    {
     "kind": "Nameplate",
     "state": "compact",
     "x": 560,
     "y": 870.0,
     "id": "Nameplate-compact"
    },
    {
     "kind": "Nameplate",
     "state": "overflow",
     "x": 560,
     "y": 1030.0,
     "id": "Nameplate-overflow"
    },
    {
     "kind": "InteractionPrompt",
     "state": "default",
     "x": 960,
     "y": 200.0,
     "id": "InteractionPrompt-default"
    },
    {
     "kind": "InteractionPrompt",
     "state": "pressed",
     "x": 960,
     "y": 320.0,
     "id": "InteractionPrompt-pressed"
    },
    {
     "kind": "InteractionPrompt",
     "state": "hold",
     "x": 960,
     "y": 440.0,
     "id": "InteractionPrompt-hold"
    },
    {
     "kind": "InteractionPrompt",
     "state": "done",
     "x": 960,
     "y": 560.0,
     "id": "InteractionPrompt-done"
    },
    {
     "kind": "InteractionPrompt",
     "state": "disabled",
     "x": 960,
     "y": 680.0,
     "id": "InteractionPrompt-disabled"
    },
    {
     "kind": "InteractionPrompt",
     "state": "gamepad",
     "x": 960,
     "y": 800.0,
     "id": "InteractionPrompt-gamepad"
    },
    {
     "kind": "InteractionPrompt",
     "state": "action",
     "x": 960,
     "y": 920.0,
     "id": "InteractionPrompt-action"
    },
    {
     "kind": "InteractionPrompt",
     "state": "overflow",
     "x": 960,
     "y": 1040.0,
     "id": "InteractionPrompt-overflow"
    },
    {
     "kind": "LootLabel",
     "state": "common",
     "x": 1270,
     "y": 190.00000000000003,
     "id": "LootLabel-common"
    },
    {
     "kind": "LootLabel",
     "state": "stack",
     "x": 1270,
     "y": 296.6666666666667,
     "id": "LootLabel-stack"
    },
    {
     "kind": "LootLabel",
     "state": "uncommon",
     "x": 1270,
     "y": 403.3333333333333,
     "id": "LootLabel-uncommon"
    },
    {
     "kind": "LootLabel",
     "state": "rare",
     "x": 1270,
     "y": 510.00000000000006,
     "id": "LootLabel-rare"
    },
    {
     "kind": "LootLabel",
     "state": "default",
     "x": 1270,
     "y": 616.6666666666667,
     "id": "LootLabel-default"
    },
    {
     "kind": "LootLabel",
     "state": "legendary",
     "x": 1270,
     "y": 723.3333333333334,
     "id": "LootLabel-legendary"
    },
    {
     "kind": "LootLabel",
     "state": "hovered",
     "x": 1270,
     "y": 830.0000000000001,
     "id": "LootLabel-hovered"
    },
    {
     "kind": "LootLabel",
     "state": "more",
     "x": 1270,
     "y": 936.6666666666667,
     "id": "LootLabel-more"
    },
    {
     "kind": "LootLabel",
     "state": "overflow",
     "x": 1270,
     "y": 1043.3333333333333,
     "id": "LootLabel-overflow"
    },
    {
     "kind": "Waypoint",
     "state": "default",
     "x": 1500,
     "y": 254.0,
     "id": "Waypoint-default"
    },
    {
     "kind": "Waypoint",
     "state": "near",
     "x": 1500,
     "y": 446.0,
     "id": "Waypoint-near"
    },
    {
     "kind": "Waypoint",
     "state": "far",
     "x": 1500,
     "y": 638.0,
     "id": "Waypoint-far"
    },
    {
     "kind": "Waypoint",
     "state": "ping",
     "x": 1500,
     "y": 830.0,
     "id": "Waypoint-ping"
    },
    {
     "kind": "Waypoint",
     "state": "danger",
     "x": 1500,
     "y": 1022.0,
     "id": "Waypoint-danger"
    },
    {
     "kind": "Waypoint",
     "state": "ally",
     "x": 1740,
     "y": 350.0,
     "id": "Waypoint-ally"
    },
    {
     "kind": "Waypoint",
     "state": "loot",
     "x": 1740,
     "y": 670.0,
     "id": "Waypoint-loot"
    },
    {
     "kind": "Waypoint",
     "state": "overflow",
     "x": 1740,
     "y": 990.0,
     "id": "Waypoint-overflow"
    },
    {
     "kind": "Waypoint",
     "state": "offscreen",
     "x": -160,
     "y": 800,
     "id": "wp-left"
    },
    {
     "kind": "Waypoint",
     "state": "offscreenRight",
     "x": 2100,
     "y": 800,
     "id": "wp-right"
    }
   ]
  ],
  "portrait": [
   390,
   844,
   [
    {
     "kind": "Nameplate",
     "state": "boss",
     "x": 195,
     "y": 100,
     "id": "Nameplate-boss"
    },
    {
     "kind": "Nameplate",
     "state": "targeted",
     "x": 195,
     "y": 190,
     "id": "Nameplate-targeted"
    },
    {
     "kind": "Nameplate",
     "state": "casting",
     "x": 195,
     "y": 280,
     "id": "Nameplate-casting"
    },
    {
     "kind": "Nameplate",
     "state": "interrupted",
     "x": 195,
     "y": 365,
     "id": "Nameplate-interrupted"
    },
    {
     "kind": "Nameplate",
     "state": "dead",
     "x": 195,
     "y": 430,
     "id": "Nameplate-dead"
    },
    {
     "kind": "InteractionPrompt",
     "state": "hold",
     "x": 195,
     "y": 520,
     "id": "InteractionPrompt-hold"
    },
    {
     "kind": "InteractionPrompt",
     "state": "disabled",
     "x": 195,
     "y": 605,
     "id": "InteractionPrompt-disabled"
    },
    {
     "kind": "LootLabel",
     "state": "hovered",
     "x": 195,
     "y": 675,
     "id": "LootLabel-hovered"
    },
    {
     "kind": "LootLabel",
     "state": "more",
     "x": 195,
     "y": 740,
     "id": "LootLabel-more"
    },
    {
     "kind": "Waypoint",
     "state": "danger",
     "x": 300,
     "y": 815,
     "id": "Waypoint-danger"
    },
    {
     "kind": "Waypoint",
     "state": "near",
     "x": 90,
     "y": 815,
     "id": "Waypoint-near"
    }
   ]
  ],
  "landscape": [
   844,
   390,
   [
    {
     "kind": "Nameplate",
     "state": "boss",
     "x": 150,
     "y": 95,
     "id": "Nameplate-boss"
    },
    {
     "kind": "Nameplate",
     "state": "targeted",
     "x": 150,
     "y": 180,
     "id": "Nameplate-targeted"
    },
    {
     "kind": "Nameplate",
     "state": "casting",
     "x": 150,
     "y": 265,
     "id": "Nameplate-casting"
    },
    {
     "kind": "Nameplate",
     "state": "dead",
     "x": 150,
     "y": 340,
     "id": "Nameplate-dead"
    },
    {
     "kind": "InteractionPrompt",
     "state": "hold",
     "x": 420,
     "y": 110,
     "id": "InteractionPrompt-hold"
    },
    {
     "kind": "InteractionPrompt",
     "state": "disabled",
     "x": 420,
     "y": 195,
     "id": "InteractionPrompt-disabled"
    },
    {
     "kind": "InteractionPrompt",
     "state": "gamepad",
     "x": 420,
     "y": 275,
     "id": "InteractionPrompt-gamepad"
    },
    {
     "kind": "Waypoint",
     "state": "near",
     "x": 420,
     "y": 365,
     "id": "Waypoint-near"
    },
    {
     "kind": "LootLabel",
     "state": "legendary",
     "x": 680,
     "y": 120,
     "id": "LootLabel-legendary"
    },
    {
     "kind": "LootLabel",
     "state": "hovered",
     "x": 680,
     "y": 185,
     "id": "LootLabel-hovered"
    },
    {
     "kind": "LootLabel",
     "state": "more",
     "x": 680,
     "y": 250,
     "id": "LootLabel-more"
    },
    {
     "kind": "Waypoint",
     "state": "danger",
     "x": 680,
     "y": 365,
     "id": "Waypoint-danger"
    }
   ]
  ]
 },
 "marks": {
  "desktop": [
   1920,
   1080,
   [
    {
     "kind": "SpeechBubble",
     "state": "default",
     "x": 330,
     "y": 250
    },
    {
     "kind": "SpeechBubble",
     "state": "short",
     "x": 760,
     "y": 250
    },
    {
     "kind": "SpeechBubble",
     "state": "shout",
     "x": 1100,
     "y": 250
    },
    {
     "kind": "SpeechBubble",
     "state": "nameless",
     "x": 1560,
     "y": 250
    },
    {
     "kind": "SpeechBubble",
     "state": "overflow",
     "x": 1500,
     "y": 470
    },
    {
     "kind": "OverheadBars",
     "state": "own",
     "x": 250,
     "y": 420,
     "id": "b0",
     "over": {
      "value": 100,
      "resource": 60,
      "resourceMax": 100
     }
    },
    {
     "kind": "OverheadBars",
     "state": "ally",
     "x": 400,
     "y": 420,
     "id": "b1",
     "over": {
      "value": 70,
      "resource": 40,
      "resourceMax": 100
     }
    },
    {
     "kind": "OverheadBars",
     "state": "enemy",
     "x": 550,
     "y": 420,
     "id": "b2",
     "over": {
      "value": 45,
      "resource": 0,
      "resourceMax": 0
     }
    },
    {
     "kind": "OverheadBars",
     "state": "neutral",
     "x": 700,
     "y": 420,
     "id": "b3",
     "over": {
      "value": 88,
      "resource": 0,
      "resourceMax": 0
     }
    },
    {
     "kind": "OverheadBars",
     "state": "enemy",
     "x": 850,
     "y": 420,
     "id": "b4",
     "over": {
      "value": 12,
      "resource": 0,
      "resourceMax": 0
     }
    },
    {
     "kind": "SelectionRing",
     "state": "default",
     "x": 180,
     "y": 620,
     "w": 90
    },
    {
     "kind": "SelectionRing",
     "state": "leader",
     "x": 480,
     "y": 620,
     "w": 90
    },
    {
     "kind": "SelectionRing",
     "state": "ally",
     "x": 780,
     "y": 620,
     "w": 90
    },
    {
     "kind": "SelectionRing",
     "state": "target",
     "x": 1080,
     "y": 620,
     "w": 90
    },
    {
     "kind": "SelectionRing",
     "state": "neutral",
     "x": 1380,
     "y": 620,
     "w": 90
    },
    {
     "kind": "SelectionRing",
     "state": "hover",
     "x": 1680,
     "y": 620,
     "w": 90
    },
    {
     "kind": "Telegraph",
     "state": "default",
     "x": 180,
     "y": 790,
     "w": 200
    },
    {
     "kind": "Telegraph",
     "state": "cone",
     "x": 480,
     "y": 790,
     "w": 200
    },
    {
     "kind": "Telegraph",
     "state": "line",
     "x": 780,
     "y": 790,
     "w": 200
    },
    {
     "kind": "Telegraph",
     "state": "ring",
     "x": 1080,
     "y": 790,
     "w": 200
    },
    {
     "kind": "Telegraph",
     "state": "impact",
     "x": 1380,
     "y": 790,
     "w": 200
    },
    {
     "kind": "Telegraph",
     "state": "ally",
     "x": 1680,
     "y": 790,
     "w": 200
    },
    {
     "kind": "PlacementGhost",
     "state": "default",
     "x": 220,
     "y": 930,
     "w": 120
    },
    {
     "kind": "PlacementGhost",
     "state": "blocked",
     "x": 560,
     "y": 930,
     "w": 120
    },
    {
     "kind": "PlacementGhost",
     "state": "partial",
     "x": 900,
     "y": 930,
     "w": 120
    },
    {
     "kind": "PathLine",
     "state": "default",
     "x": 1250,
     "y": 940,
     "w": 220
    },
    {
     "kind": "PathLine",
     "state": "invalid",
     "x": 1620,
     "y": 940,
     "w": 220
    }
   ]
  ],
  "portrait": [
   390,
   844,
   [
    {
     "kind": "SpeechBubble",
     "state": "short",
     "x": 195,
     "y": 150,
     "id": "sb-short"
    },
    {
     "kind": "SpeechBubble",
     "state": "shout",
     "x": 195,
     "y": 290,
     "id": "sb-shout"
    },
    {
     "kind": "OverheadBars",
     "state": "own",
     "x": 90,
     "y": 360,
     "id": "b0",
     "over": {
      "value": 100,
      "resource": 0,
      "resourceMax": 0
     }
    },
    {
     "kind": "OverheadBars",
     "state": "ally",
     "x": 195,
     "y": 360,
     "id": "b1",
     "over": {
      "value": 70,
      "resource": 0,
      "resourceMax": 0
     }
    },
    {
     "kind": "OverheadBars",
     "state": "enemy",
     "x": 300,
     "y": 360,
     "id": "b2",
     "over": {
      "value": 45,
      "resource": 0,
      "resourceMax": 0
     }
    },
    {
     "kind": "SelectionRing",
     "state": "leader",
     "x": 100,
     "y": 440,
     "w": 70,
     "id": "r-leader"
    },
    {
     "kind": "SelectionRing",
     "state": "target",
     "x": 290,
     "y": 440,
     "w": 70,
     "id": "r-target"
    },
    {
     "kind": "Telegraph",
     "state": "default",
     "x": 110,
     "y": 560,
     "w": 150,
     "id": "t-default"
    },
    {
     "kind": "Telegraph",
     "state": "cone",
     "x": 290,
     "y": 560,
     "w": 150,
     "id": "t-cone"
    },
    {
     "kind": "PlacementGhost",
     "state": "default",
     "x": 110,
     "y": 640,
     "w": 80,
     "id": "g-default"
    },
    {
     "kind": "PlacementGhost",
     "state": "blocked",
     "x": 290,
     "y": 640,
     "w": 80,
     "id": "g-blocked"
    },
    {
     "kind": "PathLine",
     "state": "default",
     "x": 100,
     "y": 800,
     "w": 150,
     "id": "p-default"
    }
   ]
  ],
  "landscape": [
   844,
   390,
   [
    {
     "kind": "SpeechBubble",
     "state": "short",
     "x": 150,
     "y": 110,
     "id": "sb-short"
    },
    {
     "kind": "SpeechBubble",
     "state": "shout",
     "x": 440,
     "y": 110,
     "id": "sb-shout"
    },
    {
     "kind": "OverheadBars",
     "state": "own",
     "x": 650,
     "y": 140,
     "id": "b0",
     "over": {
      "value": 100,
      "resource": 0,
      "resourceMax": 0
     }
    },
    {
     "kind": "OverheadBars",
     "state": "ally",
     "x": 720,
     "y": 140,
     "id": "b1",
     "over": {
      "value": 70,
      "resource": 0,
      "resourceMax": 0
     }
    },
    {
     "kind": "OverheadBars",
     "state": "enemy",
     "x": 790,
     "y": 140,
     "id": "b2",
     "over": {
      "value": 45,
      "resource": 0,
      "resourceMax": 0
     }
    },
    {
     "kind": "SelectionRing",
     "state": "leader",
     "x": 90,
     "y": 200,
     "w": 60,
     "id": "r-leader"
    },
    {
     "kind": "SelectionRing",
     "state": "target",
     "x": 220,
     "y": 200,
     "w": 60,
     "id": "r-target"
    },
    {
     "kind": "Telegraph",
     "state": "default",
     "x": 400,
     "y": 240,
     "w": 140,
     "id": "t-default"
    },
    {
     "kind": "Telegraph",
     "state": "cone",
     "x": 570,
     "y": 240,
     "w": 140,
     "id": "t-cone"
    },
    {
     "kind": "PlacementGhost",
     "state": "default",
     "x": 160,
     "y": 300,
     "w": 55,
     "id": "g-default"
    },
    {
     "kind": "PlacementGhost",
     "state": "blocked",
     "x": 340,
     "y": 300,
     "w": 55,
     "id": "g-blocked"
    },
    {
     "kind": "PathLine",
     "state": "default",
     "x": 540,
     "y": 330,
     "w": 130,
     "id": "p-default"
    },
    {
     "kind": "PathLine",
     "state": "invalid",
     "x": 720,
     "y": 330,
     "w": 120,
     "id": "p-invalid"
    }
   ]
  ]
 }
};
