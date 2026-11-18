// The sheet's fixture data (docs/ui.md §3.1: fixture values are data, not
// visual tuning): every fixture each member declares, as its document's
// <meta name="fixture:…"> has it, and the states the sheet adds (empty,
// overflow, disabled and the rest), each its default with what differs.
// Regenerate the documents' part when a document's fixture changes.

const banner_default = {"title":"Level up!","subtitle":"Level 12"};
const calendar_default = {"month":"March 2027","weekdays":[{"id":"1","label":"M"},{"id":"2","label":"T"},{"id":"3","label":"W"},{"id":"4","label":"T"},{"id":"5","label":"F"},{"id":"6","label":"S"},{"id":"7","label":"S"}],"days":[{"id":"d1","label":"1","today":false,"outside":false,"marked":false},{"id":"d2","label":"2","today":true,"outside":false,"marked":true}]};
const calibration_default = {"brightness":0.5,"deadZone":0.12,"offset":24,"step":"offset","taps":6};
const card_default = {"title":"Ember Blade","body":"Burns on hit. Two-handed.","image":"","badge":"New"};
const confirm_default = {"open":true,"title":"Quit to the menu?","body":"Progress since the last save is lost.","confirm":"Quit","cancel":"Keep playing","danger":true};
const creatorTools_default = {"dirty":true,"panel":"inspector","playing":false,"selected":"Lantern_03","tool":"move"};
const credits_default = {"keyArt":0,"logo":0,"offset":0,"sections":[{"id":"item-0","names":[{"id":"item-0","name":"Ada Lin","role":""}],"title":"Directed by"},{"id":"item-1","names":[{"id":"item-0","name":"Tomas Reyes","role":"Lead artist"},{"id":"item-1","name":"Mira Okafor","role":"Environment"}],"title":"Art"},{"id":"item-2","names":[{"id":"item-0","name":"Jun Sato","role":"Composer"}],"title":"Music"}]};
const dataList_default = {"loading":false,"empty":"Nothing here yet","busy":"Loading…","rows":[{"id":"a","title":"Copper ore","detail":"Mined in the north pit","value":"×12"},{"id":"b","title":"Silk","detail":"Bought in Harrow","value":"×3"}]};
const filmChapters_default = {"selected":"c2","chapters":[{"id":"c1","title":"1 · The kitchen","thumb":0,"locked":false},{"id":"c2","title":"2 · The hall","thumb":0,"locked":false},{"id":"c3","title":"3 · The stairs","thumb":0,"locked":true},{"id":"c4","title":"4 · Dawn","thumb":0,"locked":true}]};
const filmChoices_default = {"open":true,"left":0.6,"locked":false,"hotspots":[{"id":"0","x":0.58,"y":0.3,"width":0.14,"height":0.22,"label":"Take the knife"}],"choices":[{"id":"0","text":"Take the knife","enabled":true,"chosen":false,"progress":-1},{"id":"1","text":"Open the door","enabled":true,"chosen":true,"progress":-1},{"id":"2","text":"Hold to dodge","enabled":true,"chosen":false,"progress":0.45},{"id":"3","text":"Wait in the dark","enabled":false,"chosen":false,"progress":-1}]};
const filmFlowchart_default = {"view":"0 0 720 360","zoom":1,"panX":0,"panY":0,"selected":"hall","detail":"Chapter 2 · The hall","canPlay":true,"links":[{"id":"a","d":"M60 60 C 160 60, 160 60, 260 60","active":true},{"id":"b","d":"M60 60 C 160 60, 160 180, 260 180","active":false},{"id":"c","d":"M260 60 C 360 60, 360 60, 460 60","active":true},{"id":"d","d":"M260 60 C 360 60, 360 180, 460 180","active":false},{"id":"e","d":"M460 60 C 560 60, 560 60, 660 60","active":false}],"nodes":[{"id":"start","x":60,"y":60,"label":"Kitchen","icon":"flag","state":"done","shape":"square","team":""},{"id":"hall","x":260,"y":60,"label":"Hall","icon":"play","state":"done","shape":"square","team":""},{"id":"wait","x":260,"y":180,"label":"Wait","icon":"clock","state":"locked","shape":"circle","team":""},{"id":"brave","x":460,"y":60,"label":"Stairs","icon":"play","state":"current","shape":"circle","team":""},{"id":"shy","x":460,"y":180,"label":"Cellar","icon":"lock","state":"locked","shape":"circle","team":""},{"id":"end","x":660,"y":60,"label":"Dawn","icon":"crown","state":"locked","shape":"diamond","team":""}]};
const filmGallery_default = {"selected":"c2","endings":[{"id":"c1","title":"Dawn","thumb":0,"locked":false},{"id":"c2","title":"The cellar","thumb":0,"locked":false},{"id":"c3","title":"Alone","thumb":0,"locked":true},{"id":"c4","title":"The fire","thumb":0,"locked":true}]};
const gameOver_default = {"best":24100,"checkpoint":true,"left":5,"offer":true,"price":"50 gems","score":18420,"title":"You died"};
const hudEditor_default = {"showHint":true};
const keyValues_default = {"rows":[{"id":"atk","key":"Attack","value":"42","delta":"+6"},{"id":"def","key":"Defence","value":"18","delta":"-2"},{"id":"wt","key":"Weight","value":"3.5 kg","delta":""}]};
const leaderboard_default = {"league":{"demote":25,"ends":259200,"name":"Gold league","promote":10},"me":{"name":"You","rank":1284,"score":402110},"period":"This week","rows":[{"id":"item-0","name":"Nyx","rank":1,"score":982140,"texture":0,"you":false},{"id":"item-1","name":"Ferro","rank":2,"score":951300,"texture":0,"you":false},{"id":"item-2","name":"Lumen","rank":3,"score":948020,"texture":0,"you":false},{"id":"item-3","name":"Quill","rank":4,"score":902210,"texture":0,"you":false}],"selected":-1,"tab":"global"};
const loading_default = {"place":"The Sunken Archive","progress":0.62,"ready":false,"tip":"Dodging through an attack at the last moment slows time for a breath.","waits":true};
const mainMenu_default = {"canContinue":true,"keyArt":0,"logo":0,"name":"Lantern Coast","news":"Season 2 is live","saveLine":"Chapter 3 · 4h 12m","version":"1.0.0"};
const modeSelect_default = {"keyArt":0,"modes":[{"id":"item-0","line":"The campaign, alone or with a friend.","locked":false,"name":"Story","reason":"","tag":"","texture":0},{"id":"item-1","line":"Ranked fights, one life.","locked":false,"name":"Arena","reason":"","tag":"Ranked","texture":0},{"id":"item-2","line":"Hard challenges for the brave.","locked":true,"name":"Trials","reason":"Finish chapter 2","tag":"New","texture":0}],"selected":"item-0"};
const notifications_default = {"items":[{"id":"1","title":"Achievement","body":"First blood","tone":"success"}]};
const objectiveTracker_default = {"title":"The Lighthouse","objectives":[{"id":"a","label":"Find the keeper","progress":1,"done":true,"steps":0},{"id":"b","label":"Light the lamps 2/5","progress":0.4,"done":false,"steps":5}]};
const pagination_default = {"label":"Pages","atStart":false,"atEnd":false,"pages":[{"id":"1","label":"1","current":false},{"id":"2","label":"2","current":true},{"id":"3","label":"3","current":false},{"id":"gap","label":"…","gap":true},{"id":"9","label":"9","current":false}]};
const pause_default = {"canSave":true,"objective":"Reach the lighthouse","played":4520};
const results_default = {"gained":1240,"grade":"A","level":18,"rewards":[{"amount":320,"id":"item-0","name":"Gold","texture":0},{"amount":1,"id":"item-1","name":"Crate","texture":0}],"stats":[{"best":true,"count":true,"id":"item-0","name":"Score","value":48210},{"best":false,"id":"item-1","name":"Kills","value":14},{"best":false,"format":"percent","id":"item-2","name":"Accuracy","value":0.62}],"tab":"summary","title":"Victory","won":true,"xp":0.45};
const saveLoad_default = {"mode":"load","selected":"item-0","slots":[{"auto":true,"date":"Today 18:42","empty":false,"id":"item-0","place":"Old mill","played":15120,"thumb":0,"title":"Chapter 3"},{"auto":false,"date":"Yesterday","empty":false,"id":"item-1","place":"Harbour","played":9800,"thumb":0,"title":"Chapter 2"},{"auto":false,"date":"","empty":true,"id":"item-2","place":"","played":0,"thumb":0,"title":""}]};
const settings_default = {"note":""};
const splash_default = {"mark":"item-0","marks":[{"id":"item-0","name":"Studio","texture":0},{"id":"item-1","name":"Engine","texture":0}],"progress":0.4,"status":"Loading shaders"};
const starRating_default = {"label":"2 of 3 stars","stars":[{"id":"1","index":0,"lit":true},{"id":"2","index":1,"lit":true},{"id":"3","index":2,"lit":false}]};
const subtitles_default = {"lines":[{"id":"1","speaker":"Keeper","text":"The lamps went dark three nights ago."}]};
const table_default = {"label":"Standings","columns":[{"id":"n","label":"Team"},{"id":"w","label":"Won"},{"id":"p","label":"Points"}],"cells":[{"id":"r.n","row":"r","text":"Ravens"},{"id":"r.w","row":"r","text":"9"},{"id":"r.p","row":"r","text":"27"},{"id":"g.n","row":"g","text":"Gulls"},{"id":"g.w","row":"g","text":"7"},{"id":"g.p","row":"g","text":"21"}]};
const title_default = {"keyArt":0,"legal":"© Studio. All rights reserved.","logo":0,"name":"Lantern Coast","version":"1.0.0"};
const tree_default = {"label":"Scene","rows":[{"id":"w","label":"World","depth":0,"open":true,"selected":false},{"id":"p","label":"Player","depth":1,"open":null,"selected":true},{"id":"c","label":"Camera","depth":1,"open":false,"selected":false}]};
const tutorial_default = {"step":2,"steps":5,"target":"","text":"Press the dodge button just before an attack lands to slip past it.","texture":0,"title":"Dodge","waits":false,"x":64,"y":120};

export const FIXTURES: Record<string, Record<string, Record<string, unknown>>> = {
  banner: {
    default: banner_default,
    overflow: { ...banner_default, "title": "Level up! You reached the rank of Warden of the Ninefold Gate", "subtitle": "Level 120 · 3 new talents · 2 new abilities" },
  },
  calendar: {
    default: calendar_default,
    month: { ...calendar_default, "days": [{"id":"d0","label":"30","today":false,"outside":true,"marked":false},{"id":"d1","label":"31","today":false,"outside":true,"marked":false},{"id":"d2","label":"1","today":false,"outside":false,"marked":false},{"id":"d3","label":"2","today":false,"outside":false,"marked":false},{"id":"d4","label":"3","today":false,"outside":false,"marked":false},{"id":"d5","label":"4","today":false,"outside":false,"marked":true},{"id":"d6","label":"5","today":false,"outside":false,"marked":false},{"id":"d7","label":"6","today":false,"outside":false,"marked":false},{"id":"d8","label":"7","today":false,"outside":false,"marked":false},{"id":"d9","label":"8","today":false,"outside":false,"marked":false},{"id":"d10","label":"9","today":false,"outside":false,"marked":false},{"id":"d11","label":"10","today":false,"outside":false,"marked":false},{"id":"d12","label":"11","today":false,"outside":false,"marked":true},{"id":"d13","label":"12","today":false,"outside":false,"marked":false},{"id":"d14","label":"13","today":false,"outside":false,"marked":false},{"id":"d15","label":"14","today":false,"outside":false,"marked":false},{"id":"d16","label":"15","today":false,"outside":false,"marked":false},{"id":"d17","label":"16","today":true,"outside":false,"marked":true},{"id":"d18","label":"17","today":false,"outside":false,"marked":false},{"id":"d19","label":"18","today":false,"outside":false,"marked":false},{"id":"d20","label":"19","today":false,"outside":false,"marked":false},{"id":"d21","label":"20","today":false,"outside":false,"marked":false},{"id":"d22","label":"21","today":false,"outside":false,"marked":false},{"id":"d23","label":"22","today":false,"outside":false,"marked":false},{"id":"d24","label":"23","today":false,"outside":false,"marked":false},{"id":"d25","label":"24","today":false,"outside":false,"marked":false},{"id":"d26","label":"25","today":false,"outside":false,"marked":true},{"id":"d27","label":"26","today":false,"outside":false,"marked":false},{"id":"d28","label":"27","today":false,"outside":false,"marked":false},{"id":"d29","label":"28","today":false,"outside":false,"marked":false},{"id":"d30","label":"29","today":false,"outside":false,"marked":false},{"id":"d31","label":"30","today":false,"outside":false,"marked":false},{"id":"d32","label":"31","today":false,"outside":false,"marked":false},{"id":"d33","label":"1","today":false,"outside":true,"marked":false},{"id":"d34","label":"2","today":false,"outside":true,"marked":false}] },
  },
  calibration: {
    default: calibration_default,
  },
  card: {
    default: card_default,
    plain: { ...card_default, "badge": "" },
    overflow: { ...card_default, "title": "Ysolde the Unbroken, Warden of the Ninefold Gate", "body": "Burns on hit for 12 fire damage over 3 seconds. Every third hit sets the ground alight for 4 seconds. Two-handed, and too heavy to carry with a shield.", "badge": "Legendary" },
  },
  confirm: {
    default: confirm_default,
    safe: { ...confirm_default, "danger": false, "title": "Leave the lobby?", "body": "", "confirm": "Leave", "cancel": "Stay" },
    overflow: { ...confirm_default, "title": "Delete the save of the Ninefold Gate campaign for good?", "body": "Chapter 7, 61 hours played, every companion and every item in the vault. This cannot be undone, and the cloud copy is deleted with it.", "confirm": "Delete everything", "cancel": "Keep the save" },
  },
  creatorTools: {
    default: creatorTools_default,
  },
  credits: {
    default: credits_default,
    art: { ...credits_default, "keyArt": "art:key", "logo": "art:logo" },
    overflow: { ...credits_default, "sections": [{"id":"s0","title":"Directed by","names":[{"id":"n0","name":"Ada Lin","role":""},{"id":"n1","name":"Maximilian Vandenberghe-Okonkwo","role":"Lead artist"}]},{"id":"s1","title":"Art","names":[{"id":"n0","name":"Ada Lin","role":""},{"id":"n1","name":"Maximilian Vandenberghe-Okonkwo","role":"Lead artist"},{"id":"n2","name":"Mira Okafor","role":"Environment"},{"id":"n3","name":"Jun Sato","role":"Composer"}]},{"id":"s2","title":"Music","names":[{"id":"n0","name":"Ada Lin","role":""},{"id":"n1","name":"Maximilian Vandenberghe-Okonkwo","role":"Lead artist"},{"id":"n2","name":"Mira Okafor","role":"Environment"},{"id":"n3","name":"Jun Sato","role":"Composer"},{"id":"n4","name":"Abernathy-Whitcombe","role":"Principal engineer, rendering and tools"},{"id":"n5","name":"Ada Lin","role":""}]},{"id":"s3","title":"Engineering of the Ninefold Gate","names":[{"id":"n0","name":"Ada Lin","role":""},{"id":"n1","name":"Maximilian Vandenberghe-Okonkwo","role":"Lead artist"},{"id":"n2","name":"Mira Okafor","role":"Environment"},{"id":"n3","name":"Jun Sato","role":"Composer"},{"id":"n4","name":"Abernathy-Whitcombe","role":"Principal engineer, rendering and tools"},{"id":"n5","name":"Ada Lin","role":""},{"id":"n6","name":"Maximilian Vandenberghe-Okonkwo","role":"Lead artist"},{"id":"n7","name":"Mira Okafor","role":"Environment"}]},{"id":"s4","title":"Quality assurance","names":[{"id":"n0","name":"Ada Lin","role":""},{"id":"n1","name":"Maximilian Vandenberghe-Okonkwo","role":"Lead artist"},{"id":"n2","name":"Mira Okafor","role":"Environment"},{"id":"n3","name":"Jun Sato","role":"Composer"},{"id":"n4","name":"Abernathy-Whitcombe","role":"Principal engineer, rendering and tools"},{"id":"n5","name":"Ada Lin","role":""},{"id":"n6","name":"Maximilian Vandenberghe-Okonkwo","role":"Lead artist"},{"id":"n7","name":"Mira Okafor","role":"Environment"},{"id":"n8","name":"Jun Sato","role":"Composer"},{"id":"n9","name":"Abernathy-Whitcombe","role":"Principal engineer, rendering and tools"}]},{"id":"s5","title":"Special thanks","names":[{"id":"n0","name":"Ada Lin","role":""},{"id":"n1","name":"Maximilian Vandenberghe-Okonkwo","role":"Lead artist"},{"id":"n2","name":"Mira Okafor","role":"Environment"},{"id":"n3","name":"Jun Sato","role":"Composer"},{"id":"n4","name":"Abernathy-Whitcombe","role":"Principal engineer, rendering and tools"},{"id":"n5","name":"Ada Lin","role":""},{"id":"n6","name":"Maximilian Vandenberghe-Okonkwo","role":"Lead artist"},{"id":"n7","name":"Mira Okafor","role":"Environment"},{"id":"n8","name":"Jun Sato","role":"Composer"},{"id":"n9","name":"Abernathy-Whitcombe","role":"Principal engineer, rendering and tools"},{"id":"n10","name":"Ada Lin","role":""},{"id":"n11","name":"Maximilian Vandenberghe-Okonkwo","role":"Lead artist"}]}] },
  },
  dataList: {
    default: dataList_default,
    loading: { ...dataList_default, "loading": true, "rows": [] },
    empty: { ...dataList_default, "rows": [] },
    overflow: { ...dataList_default, "rows": [{"id":"r0","title":"Ysolde the Unbroken, Warden of the Ninefold Gate","detail":"Mined in the north pit beneath the Ninefold Gate during the second siege","value":"×1,250,000"},{"id":"r1","title":"Silk","detail":"Bought in Harrow","value":"×3"},{"id":"r2","title":"Copper ore","detail":"Mined in the north pit","value":"×12"},{"id":"r3","title":"Silk","detail":"Mined in the north pit beneath the Ninefold Gate during the second siege","value":"×3"},{"id":"r4","title":"Ysolde the Unbroken, Warden of the Ninefold Gate","detail":"Mined in the north pit","value":"×12"},{"id":"r5","title":"Silk","detail":"Bought in Harrow","value":"×1,250,000"},{"id":"r6","title":"Copper ore","detail":"Mined in the north pit beneath the Ninefold Gate during the second siege","value":"×12"},{"id":"r7","title":"Silk","detail":"Bought in Harrow","value":"×3"},{"id":"r8","title":"Ysolde the Unbroken, Warden of the Ninefold Gate","detail":"Mined in the north pit","value":"×12"},{"id":"r9","title":"Silk","detail":"Mined in the north pit beneath the Ninefold Gate during the second siege","value":"×3"},{"id":"r10","title":"Copper ore","detail":"Mined in the north pit","value":"×1,250,000"},{"id":"r11","title":"Silk","detail":"Bought in Harrow","value":"×3"}] },
  },
  filmChapters: {
    default: filmChapters_default,
    locked: { ...filmChapters_default, "selected": "c1", "chapters": [{"id":"c1","title":"1 · The kitchen","thumb":0,"locked":false},{"id":"c2","title":"2 · The hall","thumb":0,"locked":true},{"id":"c3","title":"3 · The stairs","thumb":0,"locked":true},{"id":"c4","title":"4 · Dawn","thumb":0,"locked":true}] },
    overflow: { ...filmChapters_default, "chapters": [{"id":"c0","title":"1 · The long night beneath the lighthouse of the northern reach","thumb":0,"locked":false},{"id":"c1","title":"2 · The hall","thumb":0,"locked":false},{"id":"c2","title":"3 · The stairs","thumb":0,"locked":false},{"id":"c3","title":"4 · The long night beneath the lighthouse of the northern reach","thumb":0,"locked":false},{"id":"c4","title":"5 · The kitchen","thumb":0,"locked":false},{"id":"c5","title":"6 · The hall","thumb":0,"locked":false},{"id":"c6","title":"7 · The long night beneath the lighthouse of the northern reach","thumb":0,"locked":false},{"id":"c7","title":"8 · Dawn","thumb":0,"locked":true},{"id":"c8","title":"9 · The kitchen","thumb":0,"locked":true},{"id":"c9","title":"10 · The long night beneath the lighthouse of the northern reach","thumb":0,"locked":true}], "selected": "c4" },
  },
  filmChoices: {
    default: filmChoices_default,
    locked: { ...filmChoices_default, "locked": true },
    timeout: { ...filmChoices_default, "left": 0.04 },
    overflow: { ...filmChoices_default, "choices": [{"id":"c0","text":"Take the knife from the table and hide it in your sleeve","enabled":true,"chosen":false,"progress":-1},{"id":"c1","text":"Open the door to the hall and call out for the keeper","enabled":true,"chosen":true,"progress":-1},{"id":"c2","text":"Hold to dodge","enabled":true,"chosen":false,"progress":0.45},{"id":"c3","text":"Wait in the dark until the footsteps pass","enabled":false,"chosen":false,"progress":-1}] },
  },
  filmFlowchart: {
    default: filmFlowchart_default,
  },
  filmGallery: {
    default: filmGallery_default,
    locked: { ...filmGallery_default, "selected": "", "endings": [{"id":"c1","title":"Dawn","thumb":0,"locked":true},{"id":"c2","title":"The cellar","thumb":0,"locked":true},{"id":"c3","title":"Alone","thumb":0,"locked":true},{"id":"c4","title":"The fire","thumb":0,"locked":true}] },
  },
  gameOver: {
    default: gameOver_default,
    noOffer: { ...gameOver_default, "offer": false, "checkpoint": false, "left": 0 },
    overflow: { ...gameOver_default, "title": "You fell at the Ninefold Gate", "score": 184320991, "best": 999999999, "price": "1,250 gems" },
  },
  hudEditor: {
    default: hudEditor_default,
  },
  keyValues: {
    default: keyValues_default,
    empty: { ...keyValues_default, "rows": [] },
    overflow: { ...keyValues_default, "rows": [{"id":"k0","key":"Critical strike chance against burning enemies","value":"42","delta":"+6"},{"id":"k1","key":"Defence","value":"1,284,000","delta":"-2"},{"id":"k2","key":"Weight","value":"3.5 kg","delta":""},{"id":"k3","key":"Attack","value":"42","delta":"+6"},{"id":"k4","key":"Critical strike chance against burning enemies","value":"1,284,000","delta":"-2"},{"id":"k5","key":"Weight","value":"3.5 kg","delta":""},{"id":"k6","key":"Attack","value":"42","delta":"+6"},{"id":"k7","key":"Defence","value":"1,284,000","delta":"-2"},{"id":"k8","key":"Critical strike chance against burning enemies","value":"3.5 kg","delta":""}] },
  },
  leaderboard: {
    default: leaderboard_default,
    empty: { ...leaderboard_default, "rows": [] },
    overflow: { ...leaderboard_default, "rows": [{"id":"p0","name":"Maximilian Vandenberghe-Okonkwo","rank":1,"score":9982140,"texture":0,"you":false},{"id":"p1","name":"Nyx","rank":2,"score":9950803,"texture":0,"you":false},{"id":"p2","name":"Abernathy-Whitcombe","rank":3,"score":9919466,"texture":0,"you":false},{"id":"p3","name":"Ferro","rank":4,"score":9888129,"texture":0,"you":false},{"id":"p4","name":"Maximilian Vandenberghe-Okonkwo","rank":5,"score":9856792,"texture":0,"you":false},{"id":"p5","name":"Nyx","rank":6,"score":9825455,"texture":0,"you":false},{"id":"p6","name":"Abernathy-Whitcombe","rank":7,"score":9794118,"texture":0,"you":false},{"id":"p7","name":"Ferro","rank":8,"score":9762781,"texture":0,"you":false},{"id":"p8","name":"Maximilian Vandenberghe-Okonkwo","rank":9,"score":9731444,"texture":0,"you":false},{"id":"p9","name":"Nyx","rank":10,"score":9700107,"texture":0,"you":true},{"id":"p10","name":"Abernathy-Whitcombe","rank":11,"score":9668770,"texture":0,"you":false},{"id":"p11","name":"Ferro","rank":12,"score":9637433,"texture":0,"you":false},{"id":"p12","name":"Maximilian Vandenberghe-Okonkwo","rank":13,"score":9606096,"texture":0,"you":false},{"id":"p13","name":"Nyx","rank":14,"score":9574759,"texture":0,"you":false},{"id":"p14","name":"Abernathy-Whitcombe","rank":15,"score":9543422,"texture":0,"you":false},{"id":"p15","name":"Ferro","rank":16,"score":9512085,"texture":0,"you":false}], "me": {"name":"You","rank":10,"score":9700107} },
  },
  loading: {
    default: loading_default,
    ready: { ...loading_default, "progress": 1, "ready": true },
    overflow: { ...loading_default, "place": "The Lighthouse of the Northern Reach, beyond the Ninefold Gate", "tip": "Dodging through an attack at the last moment slows time for a breath, long enough to step behind the attacker and strike before it turns; it works on every enemy but the largest bosses." },
  },
  mainMenu: {
    default: mainMenu_default,
    art: { ...mainMenu_default, "keyArt": "art:key", "logo": "art:logo" },
    newPlayer: { ...mainMenu_default, "canContinue": false, "saveLine": "", "news": "" },
    overflow: { ...mainMenu_default, "saveLine": "Chapter 7 · The Ninefold Gate · 61h 12m", "news": "Season 2 is live: the Ninefold Gate expansion, three new companions and a new ending" },
  },
  modeSelect: {
    default: modeSelect_default,
    art: { ...modeSelect_default, "keyArt": "art:key", "modes": modeSelect_default.modes.map((m, i) => ({ ...m, "texture": ["art:mode-story", "art:mode-arena", "art:mode-trials"][i] })) },
    overflow: { ...modeSelect_default, "modes": [{"id":"m0","line":"The campaign, alone or with a friend, through every chapter and every ending.","locked":false,"name":"Story","reason":"","tag":"","texture":0},{"id":"m1","line":"Ranked fights, one life.","locked":false,"name":"Arena","reason":"","tag":"Ranked","texture":0},{"id":"m2","line":"The campaign, alone or with a friend, through every chapter and every ending.","locked":true,"name":"Trials","reason":"Finish chapter 2 of the campaign first","tag":"New","texture":0},{"id":"m3","line":"The campaign, alone or with a friend.","locked":false,"name":"Siege of the Ninefold Gate","reason":"","tag":"","texture":0},{"id":"m4","line":"The campaign, alone or with a friend, through every chapter and every ending.","locked":false,"name":"Co-op","reason":"","tag":"Ranked","texture":0},{"id":"m5","line":"Hard challenges for the brave.","locked":true,"name":"Custom","reason":"Finish chapter 2 of the campaign first","tag":"New","texture":0}] },
  },
  notifications: {
    default: notifications_default,
    many: { ...notifications_default, "items": [{"id":"0","title":"Achievement","body":"First blood","tone":"success"},{"id":"1","title":"Friend request","body":"Maximilian Vandenberghe-Okonkwo wants to be your friend","tone":"info"},{"id":"2","title":"Connection","body":"Reconnecting to the server…","tone":"warn"},{"id":"3","title":"Purchase failed","body":"The store could not be reached. Nothing was charged.","tone":"danger"}] },
  },
  objectiveTracker: {
    default: objectiveTracker_default,
    done: { ...objectiveTracker_default, "objectives": [{"id":"a","label":"Find the keeper","progress":1,"done":true,"steps":0},{"id":"b","label":"Light the lamps 2/5","progress":1,"done":true,"steps":5}] },
    overflow: { ...objectiveTracker_default, "title": "The Lighthouse of the Northern Reach, beyond the Ninefold Gate", "objectives": [{"id":"o0","label":"Find the keeper","progress":1,"done":true,"steps":0},{"id":"o1","label":"Light every lamp along the northern reach before the tide 2/12","progress":0.4,"done":false,"steps":12},{"id":"o2","label":"Find the keeper","progress":1,"done":true,"steps":0},{"id":"o3","label":"Light every lamp along the northern reach before the tide 2/12","progress":0.4,"done":false,"steps":12},{"id":"o4","label":"Find the keeper","progress":1,"done":true,"steps":0},{"id":"o5","label":"Light every lamp along the northern reach before the tide 2/12","progress":0.4,"done":false,"steps":12}] },
  },
  pagination: {
    default: pagination_default,
    first: { ...pagination_default, "atStart": true, "pages": [{"id":"1","label":"1","current":true},{"id":"2","label":"2","current":false},{"id":"3","label":"3","current":false},{"id":"gap","label":"…","gap":true,"current":false},{"id":"9","label":"9","current":false}] },
    last: { ...pagination_default, "atEnd": true, "pages": [{"id":"1","label":"1","current":false},{"id":"2","label":"2","current":false},{"id":"3","label":"3","current":false},{"id":"gap","label":"…","gap":true,"current":false},{"id":"9","label":"9","current":true}] },
  },
  pause: {
    default: pause_default,
    noSave: { ...pause_default, "canSave": false, "objective": "" },
    overflow: { ...pause_default, "objective": "The Lighthouse of the Northern Reach, beyond the Ninefold Gate", "played": 359999 },
  },
  results: {
    default: results_default,
    lost: { ...results_default, "won": false, "grade": "D", "title": "Defeat", "gained": 120, "rewards": [] },
    overflow: { ...results_default, "title": "Flawless victory at the Ninefold Gate", "gained": 1284000, "stats": [{"best":true,"count":true,"id":"s0","name":"Score","value":48210000},{"best":false,"id":"s1","name":"Kills","value":14},{"best":false,"format":"percent","id":"s2","name":"Accuracy","value":0.62},{"best":true,"count":true,"id":"s3","name":"Critical strikes against burning enemies","value":48210},{"best":false,"id":"s4","name":"Kills","value":14000},{"best":false,"format":"percent","id":"s5","name":"Accuracy","value":0.62},{"best":true,"count":true,"id":"s6","name":"Score","value":48210000},{"best":false,"id":"s7","name":"Kills","value":14}], "rewards": [{"amount":125000,"id":"w0","name":"Gold","texture":0},{"amount":1,"id":"w1","name":"Crate","texture":0},{"amount":320,"id":"w2","name":"Ember Blade of the Ninefold Gate","texture":0},{"amount":1,"id":"w3","name":"Crate","texture":0},{"amount":320,"id":"w4","name":"Gold","texture":0},{"amount":1,"id":"w5","name":"Crate","texture":0}] },
  },
  saveLoad: {
    default: saveLoad_default,
    save: { ...saveLoad_default, "mode": "save" },
    empty: { ...saveLoad_default, "selected": "", "slots": [{"auto":false,"date":"","empty":true,"id":"e0","place":"","played":0,"thumb":0,"title":""},{"auto":false,"date":"","empty":true,"id":"e1","place":"","played":0,"thumb":0,"title":""},{"auto":false,"date":"","empty":true,"id":"e2","place":"","played":0,"thumb":0,"title":""}] },
    overflow: { ...saveLoad_default, "slots": [{"auto":true,"date":"Today 18:42","empty":false,"id":"v0","place":"The Lighthouse of the Northern Reach, beyond the Ninefold Gate","played":220000,"thumb":0,"title":"Chapter 3"},{"auto":false,"date":"Yesterday","empty":false,"id":"v1","place":"Harbour","played":223600,"thumb":0,"title":"Chapter 7 · The Ninefold Gate"},{"auto":false,"date":"Today 18:42","empty":false,"id":"v2","place":"Old mill","played":227200,"thumb":0,"title":"Chapter 3"},{"auto":false,"date":"Yesterday","empty":false,"id":"v3","place":"The Lighthouse of the Northern Reach, beyond the Ninefold Gate","played":230800,"thumb":0,"title":"Chapter 7 · The Ninefold Gate"},{"auto":false,"date":"Today 18:42","empty":false,"id":"v4","place":"Old mill","played":234400,"thumb":0,"title":"Chapter 3"},{"auto":false,"date":"Yesterday","empty":false,"id":"v5","place":"Harbour","played":238000,"thumb":0,"title":"Chapter 7 · The Ninefold Gate"},{"auto":false,"date":"Today 18:42","empty":false,"id":"v6","place":"The Lighthouse of the Northern Reach, beyond the Ninefold Gate","played":241600,"thumb":0,"title":"Chapter 3"},{"auto":false,"date":"Yesterday","empty":false,"id":"v7","place":"Harbour","played":245200,"thumb":0,"title":"Chapter 7 · The Ninefold Gate"}], "selected": "v3" },
  },
  settings: {
    default: settings_default,
    pages: { "note": "Some changes apply after a restart", "page": "video", "pending": true, "pages": [{"id":"video","name":"Video"},{"id":"audio","name":"Audio"},{"id":"controls","name":"Controls"},{"id":"accessibility","name":"Accessibility and subtitles"}], "entries": [{"id":"vsync","page":"video","name":"Vertical sync","description":"Match the display's refresh rate","kind":"toggle","value":true,"min":0,"max":1,"step":1,"options":[],"action":""},{"id":"fov","page":"video","name":"Field of view","description":"","kind":"range","value":90,"min":60,"max":120,"step":1,"options":[],"action":""},{"id":"quality","page":"video","name":"Shadow quality across the whole of the Ninefold Gate's far districts","description":"","kind":"select","value":"High","min":0,"max":0,"step":0,"options":["Low","Medium","High","Ultra"],"action":""},{"id":"motion","page":"video","name":"Reduce motion","description":"","kind":"toggle","value":false,"min":0,"max":1,"step":1,"options":[],"action":""},{"id":"jump","page":"video","name":"Jump","description":"","kind":"binding","value":"Space","min":0,"max":0,"step":0,"options":[],"action":"jump"},{"id":"master","page":"audio","name":"Master volume","description":"","kind":"range","value":80,"min":0,"max":100,"step":5,"options":[],"action":""}] },
  },
  splash: {
    default: splash_default,
    done: { ...splash_default, "progress": 1, "status": "" },
    art: { ...splash_default, "marks": [{"id":"item-0","name":"Ember & Tide","texture":"art:studio"},{"id":"item-1","name":"Engine","texture":0}] },
    overflow: { ...splash_default, "progress": 0.87, "status": "Compiling the pipeline caches for the Ninefold Gate's far districts" },
  },
  starRating: {
    default: starRating_default,
    none: { ...starRating_default, "label": "0 of 3 stars", "stars": [{"id":"1","index":0,"lit":false},{"id":"2","index":1,"lit":false},{"id":"3","index":2,"lit":false}] },
    full: { ...starRating_default, "label": "3 of 3 stars", "stars": [{"id":"1","index":0,"lit":true},{"id":"2","index":1,"lit":true},{"id":"3","index":2,"lit":true}] },
  },
  subtitles: {
    default: subtitles_default,
    overflow: { ...subtitles_default, "lines": [{"id":"1","speaker":"Keeper of the Northern Lighthouse","text":"The lamps went dark three nights ago, and since then the ships have stopped coming; if you mean to cross the gate, cross it before the tide turns."},{"id":"2","speaker":"Mira","text":"Then we go tonight."}] },
  },
  table: {
    default: table_default,
    empty: { ...table_default, "cells": [] },
    overflow: { ...table_default, "cells": [{"id":"r.n","row":"r","text":"Ravens of the Ninefold Gate"},{"id":"r.w","row":"r","text":"9"},{"id":"r.p","row":"r","text":"27"},{"id":"g.n","row":"g","text":"Gulls"},{"id":"g.w","row":"g","text":"7"},{"id":"g.p","row":"g","text":"21"},{"id":"h.n","row":"h","text":"Herons"},{"id":"h.w","row":"h","text":"1,284"},{"id":"h.p","row":"h","text":"3,852"},{"id":"k.n","row":"k","text":"Kestrels"},{"id":"k.w","row":"k","text":"0"},{"id":"k.p","row":"k","text":"0"},{"id":"o.n","row":"o","text":"Owls of the Northern Reach"},{"id":"o.w","row":"o","text":"12"},{"id":"o.p","row":"o","text":"36"}] },
  },
  title: {
    default: title_default,
    art: { ...title_default, "keyArt": "art:key", "logo": "art:logo" },
    overflow: { ...title_default, "name": "The Lighthouse of the Northern Reach", "legal": "© 2026 Ember & Tide Studio. Lantern Coast and the lighthouse mark are trademarks of Ember & Tide. All rights reserved.", "version": "1.12.4 (build 20261118.3)" },
  },
  tree: {
    default: tree_default,
    overflow: { ...tree_default, "rows": [{"id":"t0","label":"World","depth":0,"open":true,"selected":false},{"id":"t1","label":"Region of the Ninefold Gate","depth":1,"open":true,"selected":false},{"id":"t2","label":"Lighthouse","depth":2,"open":true,"selected":false},{"id":"t3","label":"Lantern_03 (the one on the north stair)","depth":3,"open":null,"selected":true},{"id":"t4","label":"Player","depth":1,"open":null,"selected":false},{"id":"t5","label":"Camera","depth":1,"open":true,"selected":false},{"id":"t6","label":"Light","depth":2,"open":true,"selected":false},{"id":"t7","label":"Particles","depth":3,"open":null,"selected":false},{"id":"t8","label":"Audio","depth":1,"open":false,"selected":false}] },
  },
  tutorial: {
    default: tutorial_default,
    last: { ...tutorial_default, "step": 5 },
    overflow: { ...tutorial_default, "title": "Dodge through the attacks of the warden", "text": "Press the dodge button just before an attack lands to slip past it. Time slows for a breath: step behind the attacker and strike before it turns. It works on every enemy but the largest bosses, whose attacks you must block." },
  },
};

export const NAMES: Record<string, string> = {"banner":"banner","calendar":"calendar","calibration":"calibration","card":"card","confirm":"confirm","creatorTools":"creator-tools","credits":"credits","dataList":"data-list","filmChapters":"film-chapters","filmChoices":"film-choices","filmFlowchart":"film-flowchart","filmGallery":"film-gallery","gameOver":"game-over","hudEditor":"hud-editor","keyValues":"key-values","leaderboard":"leaderboard","loading":"loading","mainMenu":"main-menu","modeSelect":"mode-select","notifications":"notifications","objectiveTracker":"objective-tracker","pagination":"pagination","pause":"pause","results":"results","saveLoad":"save-load","settings":"settings","splash":"splash","starRating":"star-rating","subtitles":"subtitles","table":"table","title":"title","tree":"tree","tutorial":"tutorial"};
