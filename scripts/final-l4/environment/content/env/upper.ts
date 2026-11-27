import { file } from '@origingame/engine/author';

// The pass crops zenith to horizon; the cooker fills the ground from that rim.
export default file.environment({
  source: { equirect: 'assets/env/ai-coast-upper.png' },
  hemisphere: 'upper',
  size: 256,
  samples: 256,
});
