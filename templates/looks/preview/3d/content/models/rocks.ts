import { file } from '@origingame/engine/author';

// Two boulders and a bush: smooth round forms to show the look's shading.
export default file.model({kind:'geometry',desc:{items:[
  {name:'stone',input:{kind:'material',desc:{color:[0.46,0.44,0.4],roughness:0.85}}},
  {name:'bush',input:{kind:'material',desc:{color:[0.2,0.4,0.15],roughness:0.85}}},
  {name:'boulder',input:{kind:'part',desc:{material:'stone',placements:[{transform:{position:[0,0.32,0],scale:[1.3,0.85,1]}},{transform:{position:[0.85,0.2,0.35],scale:[0.7,0.55,0.65]}}],geometry:{kind:'icosahedronGeometry',desc:{radius:0.5,detail:3}}}}},
  {name:'bush',input:{kind:'part',desc:{material:'bush',placements:[{transform:{position:[-1.1,0.3,-0.2],scale:[1.2,0.85,1.1]}},{transform:{position:[-0.55,0.25,0.45],scale:[0.8,0.65,0.8]}}],geometry:{kind:'sphereGeometry',desc:{radius:0.5,widthSegments:20,heightSegments:14}}}}},
]}});
