import { file } from '@origingame/engine/author';

export default file.quest({
  id: 'rescue',
  title: 'quest.rescue',
  category: 'side',
  globals: { wolvesFled: false },
  rewards: [{ name: 'item.gold', amount: 50, rarity: 'common' }],
  stages: [
    {
      id: 'hunt',
      objectives: [
        { id: 'wolves', text: { key: 'quest.rescue.wolves' }, target: 3, marker: [12, 0, 4] },
        { id: 'sword', text: { key: 'quest.rescue.sword' }, optional: true },
      ],
      fail: { kind: 'variable', name: 'wolvesFled' },
      next: 'report',
    },
    {
      id: 'report',
      objectives: [{
        id: 'talk',
        text: { key: 'quest.rescue.talk' },
        when: { kind: 'seen', node: 'smith.bought' },
      }],
    },
  ],
});
