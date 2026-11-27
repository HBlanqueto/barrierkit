import sys, math
import os; sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'anim'))
from an import load, spring
fr,ev,sc=load(sys.argv[1]); miny=int(sys.argv[2])//2 if len(sys.argv)>2 else 76
tu=[t for t,a in ev if a.startswith('u:')][0]
ser=[]
for t,rows,lum in fr:
    if t<tu-0.05 or t>tu+1.2: continue
    r=rows['R']; ys=[y for y in range(miny,720) if r[y][2]>=4]
    if not ys: continue
    x0=min(r[y][0] for y in ys)*2; x1=(max(r[y][1] for y in ys)+1)*2
    ser.append(((t-tu)*1000,x0,ys[0]*2,x1-x0,(ys[-1]-ys[0]+1)*2))
for s in ser[::3]: print("%.1f x %d y %d w %d h %d"%s)
def fit(k):
    a=ser[0][k]; b=ser[-1][k]
    if a==b: return None
    best=None
    for w in [x*0.5 for x in range(20,80)]:
        for z in [0.7+0.02*i for i in range(40)]:
            for d in range(0,80,2):
                e=sum((a+(b-a)*spring((t-d)/1000,w,z)-v[k])**2 for v in ser for t in [v[0]])
                if best is None or e<best[0]: best=(e,w,z,d)
    e,w,z,d=best
    return "from %d to %d w0 %.1f zeta %.2f delay %d ms rms %.2f px"%(a,b,w,z,d,math.sqrt(e/len(ser)))
ser=[s+(s[1]+s[3]/2,s[2]+s[4]/2) for s in ser]
for k,n in [(3,"w"),(4,"h"),(5,"cx"),(6,"cy")]: print(n,fit(k))
