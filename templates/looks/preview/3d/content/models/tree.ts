import { file } from '@origingame/engine/author';

// The stepped pine: a trunk under three smaller tiers, about 3.4 m tall.
export default file.model({kind:'geometry',desc:{items:[
  {name:'bark',input:{kind:'material',desc:{color:[0.33,0.2,0.1],roughness:0.9}}},
  {name:'leaves',input:{kind:'material',desc:{color:[0.15,0.42,0.2],roughness:0.85}}},
  {name:'trunk',input:{kind:'part',desc:{material:'bark',placements:[{transform:{position:[0,0.6,0]}}],geometry:{kind:'cylinderGeometry',desc:{radiusTop:0.12,radiusBottom:0.18,height:1.2,radialSegments:12}}}}},
  {name:'tier-0',input:{kind:'part',desc:{material:'leaves',placements:[{transform:{position:[0,1.55,0]}}],geometry:{kind:'coneGeometry',desc:{radius:1.15,height:1.1,radialSegments:18}}}}},
  {name:'tier-1',input:{kind:'part',desc:{material:'leaves',placements:[{transform:{position:[0,2.25,0]}}],geometry:{kind:'coneGeometry',desc:{radius:0.85,height:0.95,radialSegments:18}}}}},
  {name:'tier-2',input:{kind:'part',desc:{material:'leaves',placements:[{transform:{position:[0,2.85,0]}}],geometry:{kind:'coneGeometry',desc:{radius:0.55,height:0.85,radialSegments:18}}}}},
]}});
