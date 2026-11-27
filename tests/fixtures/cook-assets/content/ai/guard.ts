import { file } from '@origingame/engine/author';

// Patrol, raise the alarm and chase on sight, then search the last seen point.
export default file.behaviour({
  name: 'guard',
  routes: { post: [[-6, 0, -3], [-6, 0, 4], [-1, 0, 6]] },
  root: {
    kind: 'machine', initial: 'patrol', states: [
      {
        name: 'patrol',
        body: { kind: 'action', name: 'patrol post', action: { kind: 'patrol', route: 'post', wait: 1, speed: 1.5, loops: true } },
        transitions: [{ to: 'alert', checks: [{ operand: 'sees', op: 'truthy' }] }],
      },
      {
        name: 'alert',
        enter: [{ kind: 'command', name: 'alarm', args: [], cooldown: 0 }],
        body: { kind: 'action', name: 'chase', action: { kind: 'chase', key: 'target', speed: 3, reach: 1.5 } },
        transitions: [{ to: 'search', checks: [{ operand: 'sees', op: 'is', value: false }] }],
      },
      {
        name: 'search',
        body: { kind: 'action', name: 'move_to lastSeen', action: { kind: 'move-to', key: 'lastSeen', speed: 2 } },
        transitions: [
          { to: 'alert', checks: [{ operand: 'sees', op: 'truthy' }] },
          { to: 'patrol', after: 3 },
        ],
      },
    ],
  },
});
