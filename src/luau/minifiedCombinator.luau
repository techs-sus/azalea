local a,b=buffer,bit32 local c,d,e,f,g,h,i,j,k,l,m,n=string,a.create,a.writeu8,a.readu8,a.readu32,a.len,b.byteswap,b.band,b.bor,b.rshift,b.lshift,63 local o=function(o,p)local q,r=d(64),d(256)local s="ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"for t=1,64
do local u,v=t-1,c.byte(s,t)e(q,u,v)e(r,v,u)end local t=function(t,u)local
v=math.ceil(u/3)local w=v*4 local x=d(w)for y=1,v-1 do local z,A=(y-1)*4,(y-1)*3
local B=i(g(t,A))local C,D,E,F=l(B,26),j(l(B,20),n),j(l(B,14),n),j(l(B,8),n)e(x,z,f(q,C))e(x,z+1,f(q,D))e(x,z+2,f(q,E))e(x,z+3,f(q,F))end local y=u%3 if y==1 then local z=f(t,u-1)local A,B=l(z,2),j(m(z,4),n)e(x,w-4,f(q,A))e(x,w-3,f(q,B))e(x,w-2,61)e(x,w-1,61)elseif y==2 then local z=k(m(f(t,u-2),8),f(t,u-1))local
A,B,C=l(z,10),j(l(z,4),n),j(m(z,2),n)e(x,w-4,f(q,A))e(x,w-3,f(q,B))e(x,w-2,f(q,C))e(x,w-1,61)elseif y==0 and u~=0 then local z=k(m(f(t,u-3),16),m(f(t,u-2),8),f(t,u-1))local A,B,C,D=l(z,18),j(l(z,12),n),j(l(z,6),n),j(z,n)e(x,w-4,f(q,A))e(x,w-3,f(q,B))e(x,w-2,f(q,C))e(x,w-1,f(q,D))end return x end local u=t(o,p)return
game:GetService('HttpService'):JSONDecode(`\{"m":null,"t":"buffer","zbase64":"{a.readstring(u,0,h(u))}"\}`)end
local p = "%REPLACE_ME%"local q,r,s,t,u={0,10,13,34,92},d(#p),0,0,0 local v=function(v)v=m(v,1)t=k(t,l(v,u))u+=7 if u>=8 then e(r,s,t)s+=1 u-=8 t=j(m(v,(7-u)),255)end
end for w,x in utf8.codes(p)do if x>127 then local y=j(l(x,8),7)if y~=7 then v(q[y+1])end v(j(x,127))else v(j(x,255))end end local payloadBuffer=o(r,s)
