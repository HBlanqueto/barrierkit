import sys
f=sys.argv[1]; L=open(f).read().split('\n')
ev=[float(l.split()[0]) for l in L if l.endswith('start') or l.endswith('end')]
fr=[list(map(float,l.split())) for l in L if l and not l.endswith(('start','end'))]
t0=ev[0]
fr=[r for r in fr]
print(f.split('/')[-1], 'cmd %.0f ms'%((ev[1]-ev[0])*1000), 'frames',len(fr))
# per region: col 1+3*i ; print changes: time rel t0 and values when changed
for i,name in enumerate(['win','sidebar','wall']):
  prev=None; out=[]
  for r in fr:
    v=tuple(round(x) for x in r[1+3*i:4+3*i])
    if v!=prev: out.append('%+.0f:%s'%((r[0]-t0)*1000,'/'.join(map(str,v)))); prev=v
  print(' ',name,' '.join(out[:40]))
