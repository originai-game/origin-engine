import {file} from '@origingame/engine/author';

export default file.model({kind:'geometry',desc:{items:[
  {name:'ball',input:{kind:'material',desc:{color:[0.8,0.2,0.15],roughness:0.4}}},
  {name:'ball',input:{kind:'part',desc:{material:'ball',geometry:{kind:'sphereGeometry',desc:{radius:0.3}}}}},
]}});
