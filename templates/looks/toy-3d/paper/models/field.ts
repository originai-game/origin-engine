import { file } from '@origingame/engine/author';

// Sheets of card: table, cut plateau and path, with each sheet's thickness.
export default file.model({kind:'geometry',desc:{items:[
  {name:'table-card',input:{kind:'material',desc:{color:[0.2,0.32,0.15],roughness:1}}},
  {name:'path-card',input:{kind:'material',desc:{color:[0.3,0.44,0.2],roughness:1}}},
  {name:'sand-card',input:{kind:'material',desc:{color:[0.6,0.46,0.28],roughness:1}}},
  {name:'table',input:{kind:'part',desc:{material:'table-card',placements:[{transform:{position:[0,-0.02,0]}}],geometry:{kind:'boxGeometry',desc:{width:40,height:0.04,depth:40}}}}},
  {name:'plateau',input:{kind:'part',desc:{material:'path-card',placements:[{transform:{position:[0,0.06,0],quaternion:[-0.7071068,0,0,0.7071068]}}],geometry:{kind:'extrudeGeometry',desc:{shapes:[{commands:[{op:'moveTo',args:[-6,-4]},{op:'splineThru',args:[[-2,-5.5],[3,-4.8],[6.5,-3],[7,1.5],[4,4],[-1,3.6],[-5.5,4.2],[-7.5,0.5]]},{op:'closePath'}]}],depth:0.06,bevelEnabled:false,curveSegments:6}}}}},
  {name:'path',input:{kind:'part',desc:{material:'sand-card',placements:[{transform:{position:[0,0.12,0],quaternion:[-0.7071068,0,0,0.7071068]}}],geometry:{kind:'extrudeGeometry',desc:{shapes:[{commands:[{op:'moveTo',args:[-1.2,4]},{op:'splineThru',args:[[-0.2,1.5],[0.9,-0.6],[2.6,-1.6],[4.5,-2.4]]},{op:'lineTo',args:[4.8,-1.4]},{op:'splineThru',args:[[2.9,-0.5],[1.8,0.6],[0.9,2.2],[0.4,4]]},{op:'closePath'}]}],depth:0.02,bevelEnabled:false,curveSegments:6}}}}},
]}});
