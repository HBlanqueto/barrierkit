import sys
row=int(sys.argv[2]) if len(sys.argv)>2 else 35
sys.argv=['', sys.argv[1],'1250','0']
import os; exec(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'region.py')).read())
evs=[(t,a) for t,a in ev if a[0] in 'dum' and a!='end' and not a.startswith('s:')]
def segs(px,y):
    b=y*W*3; out=[]; s=None; kind=None
    for x in range(W+1):
        if x<W:
            r,g,bb=px[b+x*3:b+x*3+3]
            k='B' if (bb>200 and r<60 and g<60) else 'G' if (60<=r<=100 and abs(r-g)<4 and 10<=bb-r<=30) else 'R' if (r>200 and g<70) else None
        else: k=None
        if k!=kind:
            if kind and x-s>=2: out.append("%s%d-%d"%(kind,s+1250,x-1+1250))
            kind=k; s=x
    return ' '.join(out)
prev=None
for t,px in frames():
    st=segs(px,row)
    if st!=prev:
        last=max([e for e in evs if e[0]<=t],default=(t,'-'))
        print("%.1f ms after %s : %s"%((t-last[0])*1000,last[1][:30],st))
    prev=st
