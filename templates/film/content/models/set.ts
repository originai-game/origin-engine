import {file} from '@origingame/engine/author';

// The live ending's set; replace its models without changing the camera move.
export default file.model({kind:'geometry',desc:{items:[
  {name:'floor',input:{kind:'material',desc:{color:[0.08,0.07,0.06],roughness:0.9}}},
  {name:'stone',input:{kind:'material',desc:{color:[0.35,0.31,0.26],roughness:0.7}}},
  {name:'lamp',input:{kind:'material',desc:{color:[1,0.78,0.45],emissive:[1,0.7,0.35],emissiveIntensity:1.5}}},
  {name:'floor',input:{kind:'part',desc:{material:'floor',placements:[{transform:{position:[0,-0.05,0]}}],geometry:{kind:'boxGeometry',desc:{width:12,height:0.1,depth:12}}}}},
  {name:'pillar',input:{kind:'part',desc:{material:'stone',placements:[{transform:{position:[-1.2,1.2,0]}}],geometry:{kind:'boxGeometry',desc:{width:0.8,height:2.4,depth:0.8}}}}},
  {name:'plinth',input:{kind:'part',desc:{material:'stone',placements:[{transform:{position:[1,0.35,0.4]}}],geometry:{kind:'cylinderGeometry',desc:{radiusTop:0.6,radiusBottom:0.7,height:0.7}}}}},
  {name:'lamp',input:{kind:'part',desc:{material:'lamp',placements:[{transform:{position:[1,0.95,0.4]}}],geometry:{kind:'sphereGeometry',desc:{radius:0.25}}}}},
]}});
