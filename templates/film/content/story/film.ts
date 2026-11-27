import { file, ref } from '@origingame/engine/author';

export default file.dialogue({
  name: 'film',
  start: 'opening',
  strings: { 'opening.stay': 'Stay', 'opening.leave': 'Leave' },
  nodes: [
    {
      id: 'opening',
      segment: ref('base/segments/opening'),
      flow: { chapter: 'chapter.opening' },
      choices: [
        { id: 'stay', text: { key: 'opening.stay' }, goto: 'stay', tags: { default: '' } },
        { id: 'leave', text: { key: 'opening.leave' }, goto: 'leave' },
      ],
    },
    { id: 'stay', segment: ref('base/segments/ending'), flow: { ending: 'stay' } },
    { id: 'leave', segment: ref('base/segments/live'), flow: { ending: 'leave' } },
  ],
});
