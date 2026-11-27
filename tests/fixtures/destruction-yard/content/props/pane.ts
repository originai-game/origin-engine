import {file} from '@origingame/engine/author';
// A thin pane breaks radially and in rings around the blow.
export default file.model({kind:'geometry',desc:{items:[
  {name:'glass',input:{kind:'material',desc:{color:[0.84,0.95,0.9],roughness:0.02,transmission:1,ior:1.5,thickness:0.01,doubleSided:true}}},
  {name:'pane',input:{kind:'part',desc:{material:'glass',placements:[{transform:{position:[0,0.6,0]}}],geometry:{kind:'boxGeometry',desc:{width:1.6,height:1.2,depth:0.01}}}}}
],fracture:{style:'glass',chunks:25,rings:3,seed:9,children:0,impact:{position:[0,0.8,0],radius:0.25},interior:{color:[0.55,0.85,0.8],roughness:0.05}}}});
