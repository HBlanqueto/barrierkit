import sys, struct, zlib
base=sys.argv[1]; ox,oy=int(sys.argv[2]),int(sys.argv[3])
nr,nc,W,H=map(int,open(base+'.hdr').read().split())
size=8+(nr*W+nc*H)*3
raw=open(base+'.bin','rb').read()
ev=[(float(l.split()[0]),l.split(None,1)[1]) for l in open(base+'.ev').read().splitlines()]
def frames():
    for i in range(len(raw)//size):
        o=i*size; yield struct.unpack_from('d',raw,o)[0], raw[o+8:o+8+W*H*3]
def png(path,px):
    rows=b''.join(b'\0'+px[y*W*3:(y+1)*W*3] for y in range(H))
    c=lambda t,d: struct.pack('>I',len(d))+t+d+struct.pack('>I',zlib.crc32(t+d)&0xffffffff)
    open(path,'wb').write(b'\x89PNG\r\n\x1a\n'+c(b'IHDR',struct.pack('>IIBBBBB',W,H,8,2,0,0,0))+c(b'IDAT',zlib.compress(rows,1))+c(b'IEND',b''))
def boxes(px,pred,xr):
    xs=[];ys=[]
    for y in range(H):
        b=y*W*3
        for x in range(xr[0]-ox,xr[1]-ox):
            if pred(px[b+x*3],px[b+x*3+1],px[b+x*3+2]): xs.append(x); ys.append(y)
    return (min(xs)+ox,min(ys)+oy,max(xs)-min(xs)+1,max(ys)-min(ys)+1,len(xs)) if xs else None
