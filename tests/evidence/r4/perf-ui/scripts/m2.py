import re,sys,statistics as st
from collections import defaultdict
d=defaultdict(list); n=0
for l in open(sys.argv[1]):
    if not l.startswith('PERFUI'): continue
    n+=1
    if n<=60: continue
    head=l.rsplit(' sprites=',1)[0]
    for k,v in re.findall(r'(\S+?)=([\d.]+)',head):
        if k!='tick': d[k].append(float(v))
for k,v in d.items():
    v=sorted(v); print('  %-18s median %.4f p95 %.4f max %.1f'%(k,st.median(v),v[int(.95*len(v))],v[-1]))
