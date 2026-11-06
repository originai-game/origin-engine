// The opening: the title's Enter the keep starts a run with the party at
// the gate and the risen standing; Talk by Maren starts her conversation
// (the hero stands still while it runs), its replies branch (1 asks what
// happened, which leads on to the quest's offer; 1 again agrees), and
// agreeing starts the quest: the tracker shows it, the bag's journal holds
// it, and Maren's next words urge the party on.
import { playtest } from "@origingame/engine/playtest";
import { game, PLAY, press, script, Q_CLEAR, Q_NONE, tap, takeQuest, texts, TITLE, value, type Hero } from "./lib.ts";

const quest = takeQuest(2.0);

export default playtest({
  name: "Enter the keep, talk to Maren through her branches, and take the quest",
  commands: script(
    "0.5:get:Game:title.json",
    tap("Play", 0.8),
    "1.3:get:Game:started.json",
    "1.4:warp:maren",
    "1.6:get:hero.Hero:near.json",
    ...quest.commands,
    "2.2:get:Game:talking.json",
    "2.3:dump_ui:hello.json",
    "2.3:key_down:KeyW",
    "2.5:get:hero.Hero:still.json",
    "2.55:key_up:KeyW",
    "2.9:dump_ui:story.json",
    "3.5:dump_ui:offer.json",
    `${quest.end}:get:Game:given.json`,
    `${quest.end}:dump_ui:tracker.json`,
    press("KeyE", quest.end + 0.2),
    `${quest.end + 0.6}:dump_ui:remind.json`,
    press("KeyE", quest.end + 0.8),
    `${quest.end + 0.7}:get:Game:reminding.json`,
    `${quest.end + 1.2}:get:Game:after.json`,
    `${quest.end + 1.3}:quit`,
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(game(run, "title.json").phase, TITLE, "the game opens on its title");
    const started = game(run, "started.json");
    t.ok(started.phase === PLAY && started.quest === Q_NONE && started.slain === 0, "Enter the keep starts a run, no quest yet");
    t.equal(value<Hero>(run, "near.json").near, 1, "by Maren, Talk acts on her");
    const talking = game(run, "talking.json");
    t.ok(talking.conversation !== 0 && talking.talker === 0, "Talk started her conversation, seat 0 talking");
    t.ok(texts(run, "hello.json").some((s) => s.startsWith("Thank the gods, swords.")), "she greets the party");
    t.ok(texts(run, "hello.json").includes("What happened here?") && texts(run, "hello.json").includes("Who are you?"), "with replies to pick");
    t.ok(value<Hero>(run, "still.json").speed < 0.05, "the hero stands still while she talks");
    t.ok(texts(run, "story.json").some((s) => s.startsWith("The old warden sealed the crypt")), "1 asked what happened: she tells of the seal");
    t.ok(texts(run, "offer.json").includes("We'll do it.") && texts(run, "offer.json").includes("What does it pay?"), "then offers the quest, a reply to ask the pay among them");
    const given = game(run, "given.json");
    t.equal(given.quest, Q_CLEAR, "agreeing started the quest");
    t.equal(given.conversation, 0, "and the conversation closed");
    t.ok(texts(run, "tracker.json").includes("The Warden's Seal"), "the tracker shows the quest");
    t.ok(texts(run, "remind.json").some((s) => s.startsWith("4 of the risen still walk")), "talking again, she counts the risen left");
    t.equal(game(run, "after.json").conversation, 0, "and lets the party go");
  },
});
