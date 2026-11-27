import { file } from '@origingame/engine/author';

// The neon-vector street lantern: black faces and glowing hues.
export default file.model({kind:'geometry',desc:{items:[
  {name:'iron',input:{kind:'material',desc:{color:[0.02,0.02,0.04],roughness:1,emissive:[0.1,0.9,1],emissiveIntensity:0.6}}},
  {name:'glass',input:{kind:'material',desc:{color:[0.02,0.02,0.04],roughness:1,emissive:[1,0.3,0.8],emissiveIntensity:0.6}}},
  {name:'base',input:{kind:'part',desc:{material:'iron',placements:[{transform:{position:[0,0.12,0]}}],geometry:{kind:'boxGeometry',desc:{width:0.32,height:0.24,depth:0.32}}}}},
  {name:'post',input:{kind:'part',desc:{material:'iron',placements:[{transform:{position:[0,1.35,0]}}],geometry:{kind:'cylinderGeometry',desc:{radiusTop:0.05,radiusBottom:0.07,height:2.3,radialSegments:12}}}}},
  {name:'arm',input:{kind:'part',desc:{material:'iron',placements:[{transform:{position:[0.22,2.45,0]}}],geometry:{kind:'boxGeometry',desc:{width:0.5,height:0.06,depth:0.06}}}}},
  {name:'frame',input:{kind:'part',desc:{material:'iron',placements:[{transform:{position:[0.42,2.15,0]}}],geometry:{kind:'boxGeometry',desc:{width:0.3,height:0.06,depth:0.3}}}}},
  {name:'glass',input:{kind:'part',desc:{material:'glass',placements:[{transform:{position:[0.42,2.3,0]}}],geometry:{kind:'boxGeometry',desc:{width:0.24,height:0.26,depth:0.24}}}}},
  {name:'cap',input:{kind:'part',desc:{material:'iron',placements:[{transform:{position:[0.42,2.5,0]}}],geometry:{kind:'coneGeometry',desc:{radius:0.22,height:0.18,radialSegments:4}}}}},
]}});
