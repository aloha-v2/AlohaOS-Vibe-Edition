import os, struct, sys
path=sys.argv[1]
files=[(b"HELLO   TXT",b"Hello from the AlohaOS FAT32 filesystem!\nVirtIO Block is online.\n")]
for spec in sys.argv[2:]:
    name,source=spec.split("=",1); parts=name.upper().split("."); files.append(((parts[0][:8].ljust(8)+(parts[1][:3].ljust(3) if len(parts)>1 else "   ")).encode("ascii"),open(source,"rb").read()))
size=64*1024*1024; sectors=size//512; reserved=32; fat_count=1; spf=1024; spc=1; data_start=reserved+fat_count*spf
boot=bytearray(512); boot[0:3]=b"\xeb\x58\x90"; boot[3:11]=b"ALOHAOS "; struct.pack_into("<H",boot,11,512); boot[13]=spc; struct.pack_into("<H",boot,14,reserved); boot[16]=fat_count; struct.pack_into("<I",boot,32,sectors); struct.pack_into("<I",boot,36,spf); struct.pack_into("<I",boot,44,2); struct.pack_into("<H",boot,48,1); struct.pack_into("<H",boot,50,6); boot[64]=0x80; boot[66]=0x29; struct.pack_into("<I",boot,67,0xA10A05); boot[71:82]=b"ALOHAOSDISK"; boot[82:90]=b"FAT32   "; boot[510:512]=b"\x55\xaa"
fsinfo=bytearray(512); struct.pack_into("<I",fsinfo,0,0x41615252); struct.pack_into("<I",fsinfo,484,0x61417272); struct.pack_into("<I",fsinfo,488,0xffffffff); struct.pack_into("<I",fsinfo,492,4); struct.pack_into("<I",fsinfo,508,0xaa550000)
fat=bytearray(spf*512)
for cluster,value in ((0,0x0ffffff8),(1,0x0fffffff),(2,0x0fffffff)):
    struct.pack_into("<I",fat,cluster*4,value)
root=bytearray(512); next_cluster=3; placements=[]
for index,(name,data) in enumerate(files):
    count=max(1,(len(data)+511)//512); first=next_cluster
    for cluster in range(first,first+count): struct.pack_into("<I",fat,cluster*4,cluster+1 if cluster+1<first+count else 0x0fffffff)
    off=index*32; root[off:off+11]=name; root[off+11]=0x20; struct.pack_into("<H",root,off+26,first); struct.pack_into("<I",root,off+28,len(data)); placements.append((first,data)); next_cluster+=count
os.makedirs(os.path.dirname(path),exist_ok=True)
with open(path,"wb") as image:
    image.truncate(size); image.seek(0); image.write(boot); image.seek(512); image.write(fsinfo); image.seek(6*512); image.write(boot); image.seek(reserved*512); image.write(fat); image.seek(data_start*512); image.write(root)
    for first,data in placements: image.seek((data_start+first-2)*512); image.write(data)
print(f"Created FAT32 image: {path} ({len(files)} files)")
