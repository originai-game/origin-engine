// Draw-only linear-blend skinning, target offsets and welded normal gather.
// Independent implementation of the existing CPU garment draw algorithm.
@group(0) @binding(0) var<storage, read> input: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> topology: array<u32>;
@group(0) @binding(2) var<storage, read_write> points: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> output: array<vec4<f32>>;

fn matrix(at: u32) -> mat4x4<f32> { return mat4x4<f32>(input[at],input[at+1u],input[at+2u],input[at+3u]); }
fn index(id: vec3<u32>) -> u32 { return id.x + id.y*65535u*64u; }
fn rest(i: u32) -> vec3<f32> { return input[10u+i*4u].xyz; }

@compute @workgroup_size(64)
fn pose(@builtin(global_invocation_id) id: vec3<u32>) {
    let i=index(id);
    let count=u32(input[0].x);
    if i>=count { return; }
    let at=10u+i*4u;
    let joints=vec4<u32>(input[at+2u]);
    let weights=input[at+3u];
    let total=dot(weights,vec4<f32>(1));
    let palette_count=u32(input[0].z);
    let palette=10u+count*4u;
    var p=vec4<f32>(rest(i),1);
    if total>=1e-6 && palette_count>0u {
        var m=mat4x4<f32>();
        for(var k=0u;k<4u;k++) { m += matrix(palette+min(joints[k],palette_count-1u)*4u)*(weights[k]/total); }
        p=m*p;
    }
    let at_point=palette+palette_count*4u+i;
    let offset=input[at_point]-input[at_point+count];
    points[i]=vec4<f32>(p.xyz+(matrix(6u)*vec4<f32>(offset.xyz,0)).xyz,1);
}

@compute @workgroup_size(64)
fn scatter(@builtin(global_invocation_id) id: vec3<u32>) {
    let v=index(id);
    if v>=u32(input[0].y) { return; }
    let map=topology[3]+v*2u;
    let point=topology[map];
    let sign=bitcast<f32>(topology[map+1u]);
    let rest_normal=input[10u+point*4u+1u].xyz;
    var normal=vec3<f32>(0);
    let offset=topology[1];
    for(var k=topology[offset+point];k<topology[offset+point+1u];k++) {
        let t=4u+topology[topology[2]+k]*3u;
        let a=topology[t]; let b=topology[t+1u]; let c=topology[t+2u];
        let face=cross(points[b].xyz-points[a].xyz,points[c].xyz-points[a].xyz);
        let original=cross(rest(b)-rest(a),rest(c)-rest(a));
        normal += face*select(1.0,-1.0,dot(original,rest_normal)<0.0);
    }
    let length=dot(normal,normal);
    if length>0.0 { normal *= inverseSqrt(length); } else { normal=rest_normal; }
    let p=points[point];
    let history=u32(input[0].x)+v;
    var before=p;
    if input[1].w==0.0 { before=matrix(6u)*points[history]; }
    let current=u32(input[1].x)+v*2u;
    output[current]=p;
    output[current+1u]=vec4<f32>(normal*sign,0);
    if input[1].z!=0.0 {
        let previous=u32(input[1].y)+v*2u;
        output[previous]=before;
        output[previous+1u]=output[current+1u];
    }
    points[history]=matrix(2u)*p;
}
