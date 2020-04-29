use crate::disk::*;

#[derive(Copy, Clone)]
pub struct Directory {}

impl Directory {
    pub fn find(&self, s: &str) -> Option<u32> {
        Some(0)
    }

    // Read the superblock, if no magic number then none
    // Read First Inode
    // Construct data Blocks
    // Parse into Directory
    pub fn get_directory<'a>() -> DiskAction<'a, Option<Directory>> {
        unit(Some(Directory {}))
    }
    fn parse_directory(s: &str) -> Directory {
        Directory {}
    }
}
