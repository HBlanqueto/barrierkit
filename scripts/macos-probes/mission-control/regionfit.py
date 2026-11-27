import sys, math
sys.argv=['', sys.argv[1], '1250', '0', *sys.argv[2:]]
import os; exec(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'region.py')).read())
import os; sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'anim'))
from an import spring
red=lambda r,g,b: r>200 and g<70 and b<70
tu=[t for t,a in ev if a.startswith('u:')][0]
xr=(int(sys.argv[4]),int(sys.argv[5]))
ser=[]
for t,px in frames():
    ms=(t-tu)*1000
    if -30<ms<float(sys.argv[6] if len(sys.argv)>6 else 700):
        b=boxes(px,red,xr)
        if b: ser.append((ms,b[0],b[2],b[1]+b[3],b[0]+b[2]/2))
def fit(k):
    a=ser[0][k]; b=ser[-1][k]; best=None
    for w in [x*0.5 for x in range(16,90)]:
        for z in [0.6+0.02*i for i in range(50)]:
            for d in range(0,70,2):
                e=sum((a+(b-a)*spring((v[0]-d)/1000,w,z)-v[k])**2 for v in ser)
                if best is None or e<best[0]: best=(e,w,z,d)
    e,w,z,d=best
    return "%s->%s w0 %.1f zeta %.2f start %d ms rms %.2f"%(a,b,w,z,d,math.sqrt(e/len(ser)))
for k,n in [(1,"x0"),(2,"w"),(3,"bottom"),(4,"cx")]: print(n,fit(k))
