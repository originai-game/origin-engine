import { file } from '@origingame/engine/author';

// Four spline-cut sheets stand behind one another; the village is folded card.
export default file.model({kind:'geometry',desc:{items:[
  {name:'far-card',input:{kind:'material',desc:{color:[0.3,0.36,0.42],roughness:1}}},
  {name:'mid-card',input:{kind:'material',desc:{color:[0.25,0.36,0.24],roughness:1}}},
  {name:'near-card',input:{kind:'material',desc:{color:[0.18,0.3,0.15],roughness:1}}},
  {name:'wall',input:{kind:'material',desc:{color:[0.82,0.77,0.66],roughness:1}}},
  {name:'roof',input:{kind:'material',desc:{color:[0.62,0.27,0.18],roughness:1}}},
  {name:'sheet-far',input:{kind:'part',desc:{material:'far-card',placements:[{transform:{position:[0,0,-9]}}],geometry:{kind:'extrudeGeometry',desc:{shapes:[{commands:[{op:'moveTo',args:[-30,-3]},{op:'lineTo',args:[-30,2.52]},{op:'splineThru',args:[[-25,2.88],[-20,4.66],[-15,6.91],[-10,5.77],[-5,3.62],[0,4.13],[5,6.79],[10,7.92],[15,5.75],[20,5.28],[25,4.03],[30,2.55]]},{op:'lineTo',args:[30,-3]},{op:'closePath'}]}],depth:0.12,bevelEnabled:false,curveSegments:4}}}}},
  {name:'sheet-back',input:{kind:'part',desc:{material:'mid-card',placements:[{transform:{position:[0,0,-6]}}],geometry:{kind:'extrudeGeometry',desc:{shapes:[{commands:[{op:'moveTo',args:[-28,-3]},{op:'lineTo',args:[-28,1.5]},{op:'splineThru',args:[[-22.4,1.51],[-16.8,1.91],[-11.2,4.13],[-5.6,4.48],[0,2.15],[5.6,2.23],[11.2,4.06],[16.8,4.06],[22.4,2.21],[28,1.55]]},{op:'lineTo',args:[28,-3]},{op:'closePath'}]}],depth:0.12,bevelEnabled:false,curveSegments:4}}}}},
  {name:'sheet-mid',input:{kind:'part',desc:{material:'near-card',placements:[{transform:{position:[0,0,-2.5]}}],geometry:{kind:'extrudeGeometry',desc:{shapes:[{commands:[{op:'moveTo',args:[-26,-3]},{op:'lineTo',args:[-26,0.6]},{op:'splineThru',args:[[-20.8,0.6],[-15.6,0.66],[-10.4,1.37],[-5.2,2.91],[0,2.14],[5.2,0.85],[10.4,1.18],[15.6,2.59],[20.8,1.4],[26,0.64]]},{op:'lineTo',args:[26,-3]},{op:'closePath'}]}],depth:0.12,bevelEnabled:false,curveSegments:4}}}}},
  {name:'sheet-near',input:{kind:'part',desc:{material:'mid-card',placements:[{transform:{position:[0,0,1.8]}}],geometry:{kind:'extrudeGeometry',desc:{shapes:[{commands:[{op:'moveTo',args:[-24,-3]},{op:'lineTo',args:[-24,-0.48]},{op:'splineThru',args:[[-19.2,0.46],[-14.4,0.84],[-9.6,-0.29],[-4.8,-0.58],[0,-0.4],[4.8,0.3],[9.6,0.52],[14.4,-0.22],[19.2,-0.56],[24,-0.6]]},{op:'lineTo',args:[24,-3]},{op:'closePath'}]}],depth:0.12,bevelEnabled:false,curveSegments:4}}}}},
  {name:'house',input:{kind:'part',desc:{material:'wall',placements:[{transform:{position:[4,1.6,0]}},{transform:{position:[6.2,1.4,0.6]}},{transform:{position:[2,1.3,1]}}],geometry:{kind:'boxGeometry',desc:{width:1.6,height:1.2,depth:1.2}}}}},
  {name:'roof',input:{kind:'part',desc:{material:'roof',placements:[{transform:{position:[4,2.65,0],quaternion:[0,0.3827,0,0.9239]}},{transform:{position:[6.2,2.45,0.6],quaternion:[0,0.3827,0,0.9239]}},{transform:{position:[2,2.35,1],quaternion:[0,0.3827,0,0.9239]}}],geometry:{kind:'coneGeometry',desc:{radius:1.2,height:0.9,radialSegments:4}}}}},
  {name:'tower',input:{kind:'part',desc:{material:'wall',placements:[{transform:{position:[5,2.4,-0.6]}}],geometry:{kind:'boxGeometry',desc:{width:0.7,height:3,depth:0.7}}}}},
]}});
