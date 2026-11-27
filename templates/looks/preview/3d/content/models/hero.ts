import { file } from '@origingame/engine/author';

// The lantern keeper: a hooded traveller, 1.8 m tall, facing +Z.
export default file.model({kind:'geometry',desc:{items:[
  {name:'coat',input:{kind:'material',desc:{color:[0.16,0.36,0.16],roughness:0.85}}},
  {name:'skin',input:{kind:'material',desc:{color:[0.86,0.6,0.44],roughness:0.7}}},
  {name:'boot',input:{kind:'material',desc:{color:[0.3,0.17,0.08],roughness:0.8}}},
  {name:'iron',input:{kind:'material',desc:{color:[0.12,0.12,0.13],roughness:0.5,metalness:0.6}}},
  {name:'flame',input:{kind:'material',desc:{color:[1,0.7,0.3],emissive:[1,0.55,0.15],emissiveIntensity:4}}},
  {name:'leg-left',input:{kind:'part',desc:{material:'boot',placements:[{transform:{position:[-0.12,0.3,0]}}],geometry:{kind:'capsuleGeometry',desc:{radius:0.09,height:0.45,radialSegments:12}}}}},
  {name:'leg-right',input:{kind:'part',desc:{material:'boot',placements:[{transform:{position:[0.12,0.3,0]}}],geometry:{kind:'capsuleGeometry',desc:{radius:0.09,height:0.45,radialSegments:12}}}}},
  {name:'coat',input:{kind:'part',desc:{material:'coat',placements:[{transform:{position:[0,0.95,0]}}],geometry:{kind:'cylinderGeometry',desc:{radiusTop:0.2,radiusBottom:0.38,height:0.9,radialSegments:16}}}}},
  {name:'head',input:{kind:'part',desc:{material:'skin',placements:[{transform:{position:[0,1.55,0.04]}}],geometry:{kind:'sphereGeometry',desc:{radius:0.17,widthSegments:16,heightSegments:12}}}}},
  {name:'hood',input:{kind:'part',desc:{material:'coat',placements:[{transform:{position:[0,1.62,-0.03]}}],geometry:{kind:'coneGeometry',desc:{radius:0.25,height:0.55,radialSegments:16}}}}},
  {name:'arm',input:{kind:'part',desc:{material:'coat',placements:[{transform:{position:[0.33,1.12,0.12],quaternion:[0,0,0.3827,0.9239]}}],geometry:{kind:'capsuleGeometry',desc:{radius:0.07,height:0.4,radialSegments:10}}}}},
  {name:'lamp',input:{kind:'part',desc:{material:'iron',placements:[{transform:{position:[0.5,0.82,0.2]}}],geometry:{kind:'boxGeometry',desc:{width:0.16,height:0.22,depth:0.16}}}}},
  {name:'lamp-light',input:{kind:'part',desc:{material:'flame',placements:[{transform:{position:[0.5,0.82,0.2]}}],geometry:{kind:'sphereGeometry',desc:{radius:0.1,widthSegments:10,heightSegments:8}}}}},
]}});
