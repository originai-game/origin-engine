import { file } from '@origingame/engine/author';

// An untextured field under the neon geometry.
export default file.model({kind:'geometry',desc:{items:[
  {name:'grass',input:{kind:'material',desc:{color:[0.02,0.02,0.04],roughness:1}}},
  {name:'field',input:{kind:'part',desc:{material:'grass',placements:[{transform:{quaternion:[-0.7071068,0,0,0.7071068]}}],geometry:{kind:'planeGeometry',desc:{width:40,height:40}}}}},
]}});
