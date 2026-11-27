import {file} from '@origingame/engine/author';
export default file.model({kind:'geometry',desc:{items:[
  {name:'iron',input:{kind:'material',desc:{color:[0.03,0.035,0.04],roughness:0.5,metalness:0.6}}},
  {name:'light',input:{kind:'material',desc:{color:[1,0.85,0.6],emissive:[1,0.8,0.5],emissiveIntensity:2}}},
  {name:'pole',input:{kind:'part',desc:{material:'iron',geometry:{kind:'cylinderGeometry',desc:{radiusTop:0.06,radiusBottom:0.09,height:4.2,radialSegments:8}},placements:[{transform:{position:[0,2.1,0]}}]}}},
  {name:'head',input:{kind:'part',desc:{material:'light',geometry:{kind:'sphereGeometry',desc:{radius:0.22,widthSegments:10,heightSegments:6}},placements:[{transform:{position:[0,4.3,0]}}]}}}
]}});
