use crate::block::*;
use crate::disk::*;
use crate::file_system;
use crate::inode;
use crate::utils;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Directory {
    directory: HashMap<String, u32>,
}

impl Directory {
    pub fn find(&self, s: &str) -> Option<u32> {
        self.directory.get(s).map(|x| *x)
    }

    pub fn default() -> Directory {
        let mut h = HashMap::new();
        h.insert(".".into(), 1);
        h.insert("/".into(), 1);
        Directory { directory: h }
    }

    // Read the superblock, if no magic number then none
    // Read First Inode
    // Construct data Blocks
    // Parse into Directory
    pub fn get_directory<'a>() -> DiskAction<'a, Option<Directory>> {
        // first inode in directory inode
        let d = inode::Inode::get_inode(1);
        let d = flat_map(
            d,
            utils::lift_disk_action(Box::new(inode::Inode::get_inode_blocks)),
        );
        let d = flatten_option(d);
        let d = map(d, utils::lift(Box::new(|(_, b)| Block::blocks_to_data(b))));
        let d = map(d, utils::lift(Box::new(Directory::parse_directory)));
        flatten_option(d)
    }
    fn parse_directory(s: String) -> Option<Directory> {
        serde_json::from_str(&s).ok()
    }

    pub fn write_file_name<'a>(
        inode_number: u32,
        file_name: String,
    ) -> DiskAction<'a, Option<u32>> {
        let d = Directory::get_directory();
        let d = map(
            d,
            utils::lift(Box::new(move |mut x: Directory| {
                x.directory.insert(file_name.clone(), inode_number);
                x
            })),
        );
        let wipe = Directory::wipe_directory_blocks();
        let d = map2(d, wipe, Box::new(|a, _| a));

        let d = flat_map(
            d,
            utils::lift_disk_action(Box::new(Directory::save_directory)),
        );
        map(d, utils::lift(Box::new(move |_| inode_number)))
    }

    // prevent root directory from being remove
    pub fn remove_file_name<'a>(file_name: String) -> DiskAction<'a, Option<bool>> {
        // make sure it's not root directory
        let d = Directory::get_directory();
        let d = map(
            d,
            utils::lift(Box::new(move |mut x: Directory| {
                x.directory.remove(&file_name);
                x
            })),
        );
        let wipe = Directory::wipe_directory_blocks();
        let d = map2(d, wipe, Box::new(|a, _| a));

        let d = flat_map(
            d,
            utils::lift_disk_action(Box::new(Directory::save_directory)),
        );
        map(d, utils::lift(Box::new(move |_| true)))
    }

    pub fn save_directory<'a>(d: Directory) -> DiskAction<'a, Option<Directory>> {
        let ds = serde_json::to_string(&d)
            .ok()
            .expect("Directory failed to to_string");
        // Make sure to point inode 1 to first data block
        let blocks_data = utils::string_to_block_data_chunks(ds);
        let inode_1 = inode::Inode::get_inode(1);

        let blocks = Block::get_free_data_blocks(blocks_data.len());
        let blocks = map(
            blocks,
            Box::new(move |x| Block::set_data_blocks_data((x, blocks_data.clone()))),
        );
        let d = map2(inode_1, blocks, Box::new(inode::Inode::set_inode_blocks));
        let d = flat_map(
            d,
            utils::lift_disk_action(Box::new(file_system::write_inode_and_blocks)),
        );

        let d = flatten_option(d);
        map2(d, Directory::get_directory(), Box::new(|_, b| b))
    }
    pub fn wipe_directory_blocks<'a>() -> DiskAction<'a, Option<Vec<Block>>> {
        let d = inode::Inode::get_inode(1);
        let d = flat_map(
            d,
            utils::lift_disk_action(Box::new(inode::Inode::get_inode_blocks)),
        );
        let d = flatten_option(d);
        // Change all blocks to free, and save them
        flat_map(
            d,
            utils::lift_disk_action(Box::new(|(_, b)| Block::free_blocks(b))),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_directory_should_return_expected() {
        let disk = Disk::new("./test-files/sda1");

        let mut f = HashMap::new();
        f.insert("secret.txt".into(), 3);
        let expected = Some(Directory { directory: f });

        let (directory, _) = Directory::get_directory()(disk);
        assert_eq!(directory, expected);
    }

    #[test]
    fn save_file_should_return_expected() {
        use std::fs;
        let file_data = fs::read_to_string("./test-files/sda1").unwrap_or("".into());
        fs::write("./test-files/directory_save_test", file_data).unwrap();
        let disk = Disk::new("./test-files/directory_save_test");
        let file_name: String = "plz_work.md".into();
        let (data, disk) = Directory::write_file_name(5, file_name.clone())(disk);
        assert_eq!(data, Some(5));
        let (data, _) = Directory::get_directory()(disk);
        assert!(data.unwrap().directory.contains_key(&file_name));
    }
}
