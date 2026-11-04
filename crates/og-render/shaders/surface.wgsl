// Independent Phong tessellation (Boubekeur & Alexa, 2008).
// Technique attribution: third_party/phong-tessellation.md.
struct Vertex { pn: vec4<f32>, nuv: vec4<f32>, color: vec4<f32>, tangent: vec4<f32> }
@group(0) @binding(0) var<storage, read> coarse: array<Vertex>;
@group(0) @binding(1) var<storage, read> topology: array<u32>;
@group(0) @binding(2) var<storage, read_write> normal: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> output: array<Vertex>;

fn unit(v: vec3<f32>, fallback: vec3<f32>) -> vec3<f32> {
    let d = dot(v,v);
    if d > 0.0 { return v * inverseSqrt(d); }
    return fallback;
}
fn index(id: vec3<u32>) -> u32 { return id.x + id.y * 65535u * 64u; }

@compute @workgroup_size(64)
fn normals(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = index(id);
    if i >= topology[0] { return; }
    var n = vec3<f32>(0.0);
    let offset = topology[2];
    let faces = topology[3];
    for (var k = topology[offset+i]; k < topology[offset+i+1u]; k++) {
        let t = 8u + topology[faces+k] * 3u;
        let a = coarse[topology[t]].pn.xyz;
        let b = coarse[topology[t+1u]].pn.xyz;
        let c = coarse[topology[t+2u]].pn.xyz;
        n += cross(b-a,c-a);
    }
    normal[i] = vec4<f32>(unit(n, vec3<f32>(0,0,1)),0);
}

@compute @workgroup_size(64)
fn refine(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = index(id);
    let count = topology[6];
    if i >= topology[1]*count { return; }
    let t = 8u + (i/count)*3u;
    var ids = vec3<u32>(topology[t],topology[t+1u],topology[t+2u]);
    let sample = topology[4]+(i%count)*3u;
    var bary = bitcast<vec3<f32>>(vec3<u32>(topology[sample],topology[sample+1u],topology[sample+2u]));
    // Shared-edge samples use the same arithmetic order in either face,
    // including non-power-of-two subdivision levels.
    for (var k=0u; k<2u; k++) {
        for (var j=k+1u; j<3u; j++) {
            if ids[k]>ids[j] {
                let old_id=ids[k]; ids[k]=ids[j]; ids[j]=old_id;
                let old_weight=bary[k]; bary[k]=bary[j]; bary[j]=old_weight;
            }
        }
    }
    let a = coarse[ids.x]; let b = coarse[ids.y]; let c = coarse[ids.z];
    let ns = array<vec3<f32>,3>(normal[ids.x].xyz,normal[ids.y].xyz,normal[ids.z].xyz);
    let ps = array<vec3<f32>,3>(a.pn.xyz,b.pn.xyz,c.pn.xyz);
    var q = vec3<f32>(0);
    for (var k=0u; k<3u; k++) { if bary[k]>0.0 { q += ps[k]*bary[k]; } }
    var projected = vec3<f32>(0);
    for (var k=0u; k<3u; k++) { if bary[k]>0.0 { projected += (q - dot(q-ps[k],ns[k])*ns[k])*bary[k]; } }
    let p = mix(q,projected,0.75);
    let n = unit(ns[0]*bary.x + ns[1]*bary.y + ns[2]*bary.z,vec3<f32>(0,0,1));
    let uv = a.nuv.zw*bary.x + b.nuv.zw*bary.y + c.nuv.zw*bary.z;
    let color = a.color*bary.x + b.color*bary.y + c.color*bary.z;
    let e1 = ps[1]-ps[0]; let e2 = ps[2]-ps[0];
    let d1 = b.nuv.zw-a.nuv.zw; let d2 = c.nuv.zw-a.nuv.zw;
    let det = d1.x*d2.y-d2.x*d1.y;
    var axis = vec3<f32>(0,0,1);
    if abs(n.z)>0.9 { axis=vec3<f32>(0,1,0); }
    var tangent = unit(cross(axis,n),vec3<f32>(1,0,0));
    var sign = 1.0;
    if abs(det)>=1e-12 {
        let u = (e1*d2.y-e2*d1.y)/det;
        let v = (e2*d1.x-e1*d2.x)/det;
        tangent = unit(u-n*dot(n,u),tangent);
        if dot(cross(n,tangent),v)<0.0 { sign=-1.0; }
    }
    output[i] = Vertex(vec4<f32>(p,n.x),vec4<f32>(n.yz,uv),color,vec4<f32>(tangent,sign));
}
