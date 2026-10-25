import json,sys
TIMING={'ms','totalMs','meanMs','maxMs','lastMs','avgMs','startMs','wallMs'}
def strip(v):
    if isinstance(v,dict): return {k:strip(x) for k,x in v.items() if k not in TIMING}
    if isinstance(v,list): return [strip(x) for x in v]
    return v
a=strip(json.load(open(sys.argv[1]))); b=strip(json.load(open(sys.argv[2])))
print(sys.argv[3], 'world dump identical apart from timings' if a==b else 'WORLD DUMP DIFFERS')
