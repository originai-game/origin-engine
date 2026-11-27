import {file} from '@origingame/engine/author';

// The level's and the cutscene's set, from built-in geometry.
export default file.model({kind:'geometry',desc:{items:[
  {name:'floor',input:{kind:'material',desc:{color:[0.3,0.28,0.25],roughness:0.9}}},
  {name:'stone',input:{kind:'material',desc:{color:[0.55,0.5,0.42],roughness:0.7}}},
  {name:'floor',input:{kind:'part',desc:{material:'floor',placements:[{transform:{position:[0,-0.05,0]}}],geometry:{kind:'boxGeometry',desc:{width:12,height:0.1,depth:12}}}}},
  {name:'pillar',input:{kind:'part',desc:{material:'stone',placements:[{transform:{position:[-1.5,1.2,-1]}}],geometry:{kind:'boxGeometry',desc:{width:0.8,height:2.4,depth:0.8}}}}},
]}});
