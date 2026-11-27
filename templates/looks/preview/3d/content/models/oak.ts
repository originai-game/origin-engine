import { file } from '@origingame/engine/author';

// A broadleaf tree with three round crowns, about 3.6 m tall.
export default file.model({kind:'geometry',desc:{items:[
  {name:'bark',input:{kind:'material',desc:{color:[0.3,0.19,0.1],roughness:0.9}}},
  {name:'leaves',input:{kind:'material',desc:{color:[0.22,0.45,0.16],roughness:0.8}}},
  {name:'trunk',input:{kind:'part',desc:{material:'bark',placements:[{transform:{position:[0,0.9,0]}}],geometry:{kind:'cylinderGeometry',desc:{radiusTop:0.13,radiusBottom:0.22,height:1.8,radialSegments:14}}}}},
  {name:'crown',input:{kind:'part',desc:{material:'leaves',placements:[{transform:{position:[0,2.55,0],scale:[1.15,1,1.15]}},{transform:{position:[-0.65,2.15,0.3],scale:[0.75,0.7,0.75]}},{transform:{position:[0.6,2.25,-0.25],scale:[0.8,0.72,0.8]}}],geometry:{kind:'sphereGeometry',desc:{radius:1,widthSegments:24,heightSegments:16}}}}},
]}});
