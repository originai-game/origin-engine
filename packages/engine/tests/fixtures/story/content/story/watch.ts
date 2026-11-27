import { file } from '@origingame/engine/author';

// Objectives counted from events, with no game code.
export default file.quest({
  id: 'watch',
  title: 'quest.watch',
  stages: [{
    id: 'listen',
    objectives: [
      {
        id: 'halts', text: { key: 'quest.watch.halts' }, target: 2,
        on: 'og.StoryEvent',
        filter: {
          kind: 'binary', op: 'eq',
          left: { kind: 'variable', name: 'event.name' },
          right: { kind: 'literal', value: 'halted' },
        },
      },
      {
        id: 'hurts', text: { key: 'quest.watch.hurts' }, target: 1,
        on: 'og.BarkSaid',
        filter: {
          kind: 'binary', op: 'eq',
          left: { kind: 'variable', name: 'event.bark' },
          right: { kind: 'literal', value: 'hurt' },
        },
      },
    ],
  }],
});
