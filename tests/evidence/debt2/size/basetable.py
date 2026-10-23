import re,sys
def tw(f):
    m={};tot=0
    for l in open(f):
        t=re.match(r'^twiggy top.*: ([\d,]+) B',l)
        if t: tot=int(t.group(1).replace(',',''))
        r=re.match(r'^\| (.+?) \| (\d+) \| ([\d,]+) \|',l)
        if r: m[r.group(1)]=int(r.group(3).replace(',',''))
    return m,tot
def br(f):
    m={}
    for l in open(f):
        r=re.match(r'^\| (.+?) \| [\d,]+ \| [\d,]+ \| [\d.]+ % \| ([\d,]+) \|',l)
        if r: m[r.group(1)]=int(r.group(2).replace(',',''))
    return m
t3,T3=tw(sys.argv[1]);t2,T2=tw(sys.argv[2]);b3=br(sys.argv[3]);b2=br(sys.argv[4])
names=sorted(set(t3)|set(t2),key=lambda k:-max(t3.get(k,0),t2.get(k,0)))
f=lambda n:f"{n:,}"
print("| Crate | 3D twiggy B | 3D share | 3D brotli alone | 2D twiggy B | 2D share | 2D brotli alone |")
print("|---|---:|---:|---:|---:|---:|---:|")
rest=[0,0,0,0]
for k in names:
    v=[t3.get(k,0),b3.get(k,0),t2.get(k,0),b2.get(k,0)]
    if max(v[0],v[2])<20000:
        rest=[a+b for a,b in zip(rest,v)];continue
    print(f"| {k} | {f(v[0])} | {100*v[0]/T3:.2f} % | {f(v[1])} | {f(v[2])} | {100*v[2]/T2:.2f} % | {f(v[3])} |")
print(f"| (the rest, under 20,000 B) | {f(rest[0])} | {100*rest[0]/T3:.2f} % | {f(rest[1])} | {f(rest[2])} | {100*rest[2]/T2:.2f} % | {f(rest[3])} |")
print(f"| **module** | **{f(T3)}** | 100 % | | **{f(T2)}** | 100 % | |")
