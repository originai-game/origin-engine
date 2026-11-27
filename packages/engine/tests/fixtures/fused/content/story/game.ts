import { file, input, ref } from '@origingame/engine/author';

export default file.dialogue({
  yarn: input.text('sources/story/game.yarn'),
  assets: {
    'base/segments/intro': ref('base/segments/intro'),
    'base/segments/level': ref('base/segments/level'),
    'base/segments/cutscene': ref('base/segments/cutscene'),
    'base/segments/arcade': ref('base/segments/arcade'),
    'base/segments/ending': ref('base/segments/ending'),
    'base/segments/retry': ref('base/segments/retry'),
  },
});
