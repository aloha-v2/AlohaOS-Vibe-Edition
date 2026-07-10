import os, struct, sys

path = sys.argv[1]
size = 64 * 1024 * 1024
sectors = size // 512
reserved = 32
fat_count = 1
sectors_per_fat = 1024
sectors_per_cluster = 1
data_start = reserved + fat_count * sectors_per_fat

boot = bytearray(512)
boot[0:3] = b'\xeb\x58\x90'
boot[3:11] = b'ALOHAOS '
struct.pack_into('<H', boot, 11, 512)
boot[13] = sectors_per_cluster
struct.pack_into('<H', boot, 14, reserved)
boot[16] = fat_count
struct.pack_into('<H', boot, 17, 0)
struct.pack_into('<H', boot, 19, 0)
boot[21] = 0xF8
struct.pack_into('<H', boot, 22, 0)
struct.pack_into('<I', boot, 32, sectors)
struct.pack_into('<I', boot, 36, sectors_per_fat)
struct.pack_into('<I', boot, 44, 2)
struct.pack_into('<H', boot, 48, 1)
struct.pack_into('<H', boot, 50, 6)
boot[64] = 0x80
boot[66] = 0x29
struct.pack_into('<I', boot, 67, 0xA10A05)
boot[71:82] = b'ALOHAOSDISK'
boot[82:90] = b'FAT32   '
boot[510:512] = b'\x55\xaa'

fsinfo = bytearray(512)
struct.pack_into('<I', fsinfo, 0, 0x41615252)
struct.pack_into('<I', fsinfo, 484, 0x61417272)
struct.pack_into('<I', fsinfo, 488, 0xffffffff)
struct.pack_into('<I', fsinfo, 492, 4)
struct.pack_into('<I', fsinfo, 508, 0xaa550000)

fat = bytearray(sectors_per_fat * 512)
for cluster, value in ((0, 0x0ffffff8), (1, 0x0fffffff), (2, 0x0fffffff), (3, 0x0fffffff)):
    struct.pack_into('<I', fat, cluster * 4, value)

text = b'Hello from the AlohaOS FAT32 filesystem!\nVirtIO Block is online.\n'
root = bytearray(512)
root[0:11] = b'HELLO   TXT'
root[11] = 0x20
struct.pack_into('<H', root, 26, 3)
struct.pack_into('<I', root, 28, len(text))

os.makedirs(os.path.dirname(path), exist_ok=True)
with open(path, 'wb') as image:
    image.truncate(size)
    image.seek(0); image.write(boot)
    image.seek(512); image.write(fsinfo)
    image.seek(6 * 512); image.write(boot)
    image.seek(reserved * 512); image.write(fat)
    image.seek(data_start * 512); image.write(root)
    image.seek((data_start + 1) * 512); image.write(text)
print(f'Created FAT32 image: {path}')
