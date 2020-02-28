use core::fmt;
use shim::const_assert_size;

use crate::traits::BlockDevice;
use crate::vfat::Error;

#[repr(C, packed)]
pub struct BiosParameterBlock {
    __jmp_short_nop: [u8; 3],
    oem_identifier: u64,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    num_reserved_sectors: u16,
    num_FATs: u8,
    max_dir_entries: u16,
    total_logical_sectors: u16,
    fat_id: u8,
    __sectors_per_FAT: u16,
    sectors_per_track: u16,
    num_heads: u16,
    num_hidden_sectors: u32,
    total_logical_sectors_alt: u32,
    sectors_per_FAT: u32,
    flags: u16,
    FAT_version_num: u16,
    root_cluster_num: u32,
    fsinfo_sector_num: u16,
    backup_bootsector_sector_num: u16,
    _reserved: [u8; 12],
    drive_num: u8,
    windowsNT_flags: u8,
    signature: u8,
    volume_id: u32,
    volume_label_string: [u8; 11],
    sys_id_string: u64,
    boot_code: [u8; 420],
    bootable_partition_signature: u16
}

const_assert_size!(BiosParameterBlock, 512);

impl BiosParameterBlock {
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(mut device: T, sector: u64) -> Result<BiosParameterBlock, Error> {
        let buf = &mut [0u8; 512];
        // doc comment doesn't say to do this but imma do it to be safe
        if let Err(e) = device.read_sector(sector, buf) {
            return Err(Error::Io(e));
        }
        let bpb = unsafe{ core::mem::transmute::<[u8; 512], BiosParameterBlock>(*buf) };

        //why is this the 'bootable partition signature' and not the 'signature'
        if bpb.bootable_partition_signature != 0xAA55 {
            return Err(Error::BadSignature);
        }
        Ok(bpb)
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BiosParameterBlock")
            .field("bytes_per_sector", &self.bytes_per_sector)
            .field("sectors_per_cluster", &self.sectors_per_cluster)
            .field("sectors_per_track", &self.sectors_per_track)
            .field("sectors_per_FAT", &self.sectors_per_FAT)
            .field("root_cluster_num", &self.root_cluster_num)
            .field("bootable_partition_signature", &self.bootable_partition_signature)
            .finish()
    }
}
