import {file} from '@origingame/engine/author';
// A running bond anchored at its foot and ends breaks along the mortar.
export default file.model({kind:'geometry',desc:{items:[
  {name:'brick',input:{kind:'material',desc:{color:[0.58,0.3,0.22],roughness:0.9}}},
  {name:'wall',input:{kind:'part',desc:{material:'brick',placements:[{transform:{position:[0,1.2,0]}}],geometry:{kind:'boxGeometry',desc:{width:5,height:2.4,depth:0.3}}}}}
],fracture:{style:'brick',brick:[0.25,0.1],chunks:250,seed:11,children:32,anchor:{kind:'supports',bottom:true,ends:true},interior:{color:[0.66,0.38,0.28],roughness:1},mortar:{color:[0.66,0.63,0.57],roughness:1}}}});
