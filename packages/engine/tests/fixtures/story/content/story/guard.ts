import { file, input, ref } from '@origingame/engine/author';

export default file.dialogue({
  yarn: input.text('sources/story/guard.yarn'),
  assets: { 'vo/guard_halt.wav': ref('base/vo/guard_halt') },
});
