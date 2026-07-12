//! Small read-only FAT32 filesystem for the AlohaOS shell.

use crate::{framebuffer, sync::IrqSpinLock, virtio_blk};

#[derive(Clone, Copy)]
struct Volume {
    sectors_per_cluster: u32,
    reserved_sectors: u32,
    fat_count: u32,
    sectors_per_fat: u32,
    root_cluster: u32,
    fat_start: u32,
    data_start: u32,
}

static VOLUME: IrqSpinLock<Option<Volume>> = IrqSpinLock::new(None);

pub fn init() -> bool {
    let mut sector = [0u8; 512];
    if !virtio_blk::read_sector(0, &mut sector)
        || sector[510] != 0x55
        || sector[511] != 0xaa
    {
        return false;
    }

    let bytes_per_sector = le16(&sector, 11);
    let sectors_per_cluster = sector[13] as u32;
    let reserved_sectors = le16(&sector, 14) as u32;
    let fat_count = sector[16] as u32;
    let sectors_per_fat = le32(&sector, 36);
    let root_cluster = le32(&sector, 44);
    if bytes_per_sector != 512
        || sectors_per_cluster == 0
        || sectors_per_fat == 0
        || root_cluster < 2
    {
        return false;
    }

    *VOLUME.lock() = Some(Volume {
        sectors_per_cluster,
        reserved_sectors,
        fat_count,
        sectors_per_fat,
        root_cluster,
        fat_start: reserved_sectors,
        data_start: reserved_sectors + fat_count * sectors_per_fat,
    });
    true
}

pub fn is_mounted() -> bool {
    VOLUME.lock().is_some()
}

fn mounted_volume() -> Option<Volume> {
    *VOLUME.lock()
}

pub fn list_root() {
    let Some(volume) = mounted_volume() else {
        framebuffer::write_line("FAT32 NOT MOUNTED");
        return;
    };
    walk_directory(volume, |entry| {
        write_short_name(entry);
        if entry[11] & 0x10 != 0 {
            framebuffer::write_text("/");
        }
        framebuffer::write_byte(b'\n');
        false
    });
}

pub fn cat(name: &[u8]) {
    let Some(volume) = mounted_volume() else {
        framebuffer::write_line("FAT32 NOT MOUNTED");
        return;
    };
    let Some(target) = make_short_name(name) else {
        framebuffer::write_line("USE 8.3 FILE NAME");
        return;
    };

    let mut found = None;
    walk_directory(volume, |entry| {
        if entry[..11] == target {
            let high = le16(entry, 20) as u32;
            let low = le16(entry, 26) as u32;
            found = Some(((high << 16) | low, le32(entry, 28)));
            true
        } else {
            false
        }
    });

    let Some((mut cluster, mut remaining)) = found else {
        framebuffer::write_line("FILE NOT FOUND");
        return;
    };
    let mut sector = [0u8; 512];
    while cluster >= 2 && cluster < 0x0fff_fff8 && remaining != 0 {
        let first = cluster_sector(volume, cluster);
        for offset in 0..volume.sectors_per_cluster {
            if !virtio_blk::read_sector((first + offset) as u64, &mut sector) {
                framebuffer::write_line("DISK READ ERROR");
                return;
            }
            let count = remaining.min(512) as usize;
            for &byte in &sector[..count] {
                if byte == b'\n'
                    || byte == b'\r'
                    || byte == b'\t'
                    || byte.is_ascii_graphic()
                    || byte == b' '
                {
                    framebuffer::write_byte(if byte == b'\r' {
                        b'\n'
                    } else if byte == b'\t' {
                        b' '
                    } else {
                        byte
                    });
                }
            }
            remaining -= count as u32;
            if remaining == 0 {
                break;
            }
        }
        cluster = next_cluster(volume, cluster).unwrap_or(0x0fff_ffff);
    }
    framebuffer::write_byte(b'\n');
}

fn walk_directory(volume: Volume, mut visitor: impl FnMut(&[u8; 32]) -> bool) {
    let mut cluster = volume.root_cluster;
    let mut sector = [0u8; 512];
    for _ in 0..128 {
        let first = cluster_sector(volume, cluster);
        for offset in 0..volume.sectors_per_cluster {
            if !virtio_blk::read_sector((first + offset) as u64, &mut sector) {
                return;
            }
            for raw in sector.chunks_exact(32) {
                if raw[0] == 0 {
                    return;
                }
                if raw[0] == 0xe5 || raw[11] == 0x0f || raw[11] & 0x08 != 0 {
                    continue;
                }
                let entry: &[u8; 32] = raw.try_into().unwrap();
                if visitor(entry) {
                    return;
                }
            }
        }
        let Some(next) = next_cluster(volume, cluster) else {
            return;
        };
        if next >= 0x0fff_fff8 {
            return;
        }
        cluster = next;
    }
}

fn next_cluster(volume: Volume, cluster: u32) -> Option<u32> {
    let offset = cluster * 4;
    let mut sector = [0u8; 512];
    virtio_blk::read_sector(
        (volume.fat_start + offset / 512) as u64,
        &mut sector,
    )
    .then(|| le32(&sector, (offset % 512) as usize) & 0x0fff_ffff)
}

fn cluster_sector(volume: Volume, cluster: u32) -> u32 {
    volume.data_start + (cluster - 2) * volume.sectors_per_cluster
}

fn write_short_name(entry: &[u8; 32]) {
    for &byte in &entry[..8] {
        if byte != b' ' {
            framebuffer::write_byte(byte);
        }
    }
    if entry[8] != b' ' {
        framebuffer::write_byte(b'.');
        for &byte in &entry[8..11] {
            if byte != b' ' {
                framebuffer::write_byte(byte);
            }
        }
    }
}

fn make_short_name(name: &[u8]) -> Option<[u8; 11]> {
    let mut output = [b' '; 11];
    let mut base = 0usize;
    let mut ext = 8usize;
    let mut dotted = false;
    for &byte in name {
        if byte == b'.' {
            if dotted {
                return None;
            }
            dotted = true;
            continue;
        }
        if !byte.is_ascii_alphanumeric() && byte != b'_' {
            return None;
        }
        if dotted {
            if ext == 11 {
                return None;
            }
            output[ext] = byte.to_ascii_uppercase();
            ext += 1;
        } else {
            if base == 8 {
                return None;
            }
            output[base] = byte.to_ascii_uppercase();
            base += 1;
        }
    }
    (base > 0).then_some(output)
}

fn le16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn le32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}
