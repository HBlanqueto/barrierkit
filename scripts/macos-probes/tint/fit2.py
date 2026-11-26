import sys, subprocess, collections, itertools
T='/private/tmp/claude-501/-Users-humbe--claude-spares-spare-wxcbfZ/815469d1-10d1-46d0-9155-bfb2393396d4/scratchpad/tint/'
C=['red','green','blue','white','black','gray','orange','navy','ice']
W={'red':[255,0,0],'green':[0,255,2],'blue':[2,1,255],'white':[255]*3,'black':[0]*3,'gray':[128]*3,'orange':[255,128,2],'navy':[38,81,160],'ice':[201,220,255]}
rows=collections.defaultdict(dict)
for c in C:
  for l in subprocess.run([T+'an',c+'-on'],capture_output=True,text=True).stdout.splitlines():
    t,ap,st,lab,*v=l.split(); rows[(lab,ap,st)][c]=list(map(float,v))
def solve(A,b):
  n=len(A[0]); M=[[sum(A[k][i]*A[k][j] for k in range(len(A))) for j in range(n)]+[sum(A[k][i]*b[k] for k in range(len(A)))] for i in range(n)]
  for i in range(n):
    p=max(range(i,n),key=lambda r:abs(M[r][i])); M[i],M[p]=M[p],M[i]
    if abs(M[i][i])<1e-12: continue
    for r in range(n):
      if r!=i:
        f=M[r][i]/M[i][i]; M[r]=[x-f*y for x,y in zip(M[r],M[i])]
  return [M[i][n]/M[i][i] if abs(M[i][i])>1e-12 else 0 for i in range(n)]
def fit(feat,d):
  # shared coefficients across channels on features feat(w,ch); per-channel? use shared
  A=[];b=[]
  for c in C:
    for ch in range(3): A.append(feat(W[c],ch)); b.append(d[c][ch])
  x=solve(A,b); e=[bb-sum(p*q for p,q in zip(a,x)) for a,bb in zip(A,b)]
  return x,(sum(v*v for v in e)/len(e))**.5,max(map(abs,e))
Y=lambda w:0.2126*w[0]+0.7152*w[1]+0.0722*w[2]
M=lambda w:sum(w)/3
models={
 'k*wall':lambda w,ch:[1,w[ch]],
 'L709+chroma':lambda w,ch:[1,Y(w),w[ch]-Y(w)],
 'mean+chroma':lambda w,ch:[1,M(w),w[ch]-M(w)],
 'max+chroma':lambda w,ch:[1,max(w),w[ch]-M(w)],
}
for key in [('windowBackground','dark','active'),('headerView','dark','active'),('sidebar','dark','active'),('sidebar','dark','inactive'),('sidebar','aqua','active'),('sidebar','aqua','inactive'),('hudWindow','dark','active'),('split-sidebar','dark','active')]:
  d=rows[key]; s=' '.join('%s:%s rms%.2f/%.1f'%(m,[round(v,4) for v in fit(f,d)[0]],*fit(f,d)[1:]) for m,f in models.items())
  print(key,s)
