import sys
from lc import load, px
fr, ev = load(sys.argv[1])
evs=[(t,a) for t,a in ev if a[0] in 'mk']
prev=None; pt=None
for t,rows,cols in fr:
    st=(px(rows[0],int(sys.argv[2])),px(rows[1],int(sys.argv[3])),px(cols[0],int(sys.argv[4])))
    if st!=prev:
        last=max([e for e in evs if e[0]<=t], default=(0,'-'))
        print("%.1f after %s (dt %.1f) ring %s cap %s col %s"%((t-last[0])*1000,last[1],(t-pt)*1000 if pt else 0,*st))
    prev=st; pt=t
