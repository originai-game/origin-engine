import {file} from '@origingame/engine/author';
// A concrete shaft loses its middle band; the top falls onto its stump.
export default file.model({kind:'geometry',desc:{items:[
  {name:'concrete',input:{kind:'material',desc:{color:[0.62,0.61,0.58],roughness:0.85}}},
  {name:'shaft',input:{kind:'part',desc:{material:'concrete',placements:[{transform:{position:[0,1.5,0]}}],geometry:{kind:'cylinderGeometry',desc:{radiusTop:0.25,radiusBottom:0.25,height:3,radialSegments:20}}}}}
],fracture:{style:'concrete',chunks:44,seed:5,children:5,interior:{color:[0.5,0.49,0.46],roughness:0.95}}}});
