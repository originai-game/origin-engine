import { file } from '@origingame/engine/author';

// The final L4 pass supplies the pinned coast panorama at this exact path.
export default file.environment({
  source: { equirect: 'assets/env/ai-coast.png' },
  size: 256,
  samples: 256,
});
