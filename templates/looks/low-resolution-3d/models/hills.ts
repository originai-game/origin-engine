import { file } from '@origingame/engine/author';

// Low-resolution-3d's faceted far ridge: coarse spheres and a village.
export default file.model({kind:'geometry',desc:{items:[
  {name:'hill',input:{kind:'material',desc:{color:[0.24,0.36,0.3],roughness:1}}},
  {name:'far-hill',input:{kind:'material',desc:{color:[0.3,0.38,0.45],roughness:1}}},
  {name:'wall',input:{kind:'material',desc:{color:[0.75,0.7,0.6],roughness:0.9}}},
  {name:'roof',input:{kind:'material',desc:{color:[0.55,0.22,0.15],roughness:0.9}}},
  {name:'hill-0',input:{kind:'part',desc:{material:'far-hill',placements:[{transform:{position:[-14,-2,-8],scale:[12,6,4]}}],geometry:{kind:'sphereGeometry',desc:{radius:1,widthSegments:6,heightSegments:4}}}}},
  {name:'hill-1',input:{kind:'part',desc:{material:'far-hill',placements:[{transform:{position:[10,-2.5,-9],scale:[15,7.5,4]}}],geometry:{kind:'sphereGeometry',desc:{radius:1,widthSegments:6,heightSegments:4}}}}},
  {name:'hill-2',input:{kind:'part',desc:{material:'hill',placements:[{transform:{position:[-3,-3,-2],scale:[13,5,4]}}],geometry:{kind:'sphereGeometry',desc:{radius:1,widthSegments:6,heightSegments:4}}}}},
  {name:'hill-3',input:{kind:'part',desc:{material:'hill',placements:[{transform:{position:[16,-3,-1],scale:[9,4.5,4]}}],geometry:{kind:'sphereGeometry',desc:{radius:1,widthSegments:6,heightSegments:4}}}}},
  {name:'house',input:{kind:'part',desc:{material:'wall',placements:[{transform:{position:[4,1.6,0]}},{transform:{position:[6.2,1.4,0.6]}},{transform:{position:[2,1.3,1]}}],geometry:{kind:'boxGeometry',desc:{width:1.6,height:1.2,depth:1.2}}}}},
  {name:'roof',input:{kind:'part',desc:{material:'roof',placements:[{transform:{position:[4,2.65,0],quaternion:[0,0.3827,0,0.9239]}},{transform:{position:[6.2,2.45,0.6],quaternion:[0,0.3827,0,0.9239]}},{transform:{position:[2,2.35,1],quaternion:[0,0.3827,0,0.9239]}}],geometry:{kind:'coneGeometry',desc:{radius:1.2,height:0.9,radialSegments:4}}}}},
  {name:'tower',input:{kind:'part',desc:{material:'wall',placements:[{transform:{position:[5,2.4,-0.6]}}],geometry:{kind:'boxGeometry',desc:{width:0.7,height:3,depth:0.7}}}}},
]}});
