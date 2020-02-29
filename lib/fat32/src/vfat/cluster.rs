#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Hash)]
pub struct Cluster(u32);

impl From<u32> for Cluster {
    fn from(raw_num: u32) -> Cluster {
        Cluster(raw_num & !(0xF << 28))
    }
}

// TODO: Implement any useful helper methods on `Cluster`.
impl Cluster {
    pub fn get_cluster_number(&self) -> u32 {
        self.0
    }
    pub fn sector_from_cluster(&self, first_data_sector: u64, sec_per_cluster: u64) -> u64 {
        ((self.0 as u64 - 2) * sec_per_cluster + first_data_sector) 
    }
}