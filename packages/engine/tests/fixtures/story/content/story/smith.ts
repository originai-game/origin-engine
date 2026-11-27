import { file, input, ref } from '@origingame/engine/author';

export default file.dialogue({
  yarn: input.text('sources/story/smith.yarn'),
  assets: { 'art/smith.png': ref('base/art/smith') },
});
