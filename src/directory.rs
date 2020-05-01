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
        let d = map(d, utils::lift(Box::new(Block::blocks_to_data)));
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
        // Read Directory
        // Add new key value pair of i and file_name to directory
        // Convert the Directory into a string
        // Chunk up string into max amount vec string array
        // Now the complex part :/
        // Option 1
        // Free up all blocks dictionary is currently on
        // then get the amount of free blocks needed for vec string
        // Write it out
        let d = Directory::get_directory();
        let d = map(
            d,
            utils::lift(Box::new(move |mut x: Directory| {
                x.directory.insert(file_name.clone(), inode_number);
                x
            })),
        );
        let wipe = Directory::wipe_directory_blocks();
        let d = map2(d, wipe, Box::new(|a, b| a));
        let d = map(d, utils::lift(Box::new(Directory::save_directory)));
        map(d, utils::lift(Box::new(move |_| inode_number)))
    }

    pub fn save_directory<'a>(d: Directory) -> DiskAction<'a, Option<Directory>> {
        let ds = serde_json::to_string(&d).ok().unwrap_or("".into());
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
        let d = flat_map(
            d,
            utils::lift_disk_action(Box::new(|blocks| {
                let blocks = blocks
                    .into_iter()
                    .map(|x| x.free())
                    .map(|x| Block::write_block(x))
                    .collect();
                sequence(blocks)
            })),
        );
        map(d, utils::lift(Box::new(utils::remove_options)))
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
}
