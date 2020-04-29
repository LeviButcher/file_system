use crate::block::*;
use crate::disk::*;
use crate::file_system;
use crate::line_handler;
use serde::{Deserialize, Serialize};

static INODE_TABLE_SIZE: u32 = 5;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Inode {
    pub number: u32,
    pub start_block: Option<u32>, // None means inode is free
}

// First Inode is associated with the directory
// Inode table spans across two blocks
// Inode table blocks take up 10% of available blocks

impl Inode {
    fn get_inode_table<'a>() -> DiskAction<'a, Option<Vec<Inode>>> {
        let d = SuperBlock::get_super_block();
        let d = flat_map(
            d,
            file_system::lift_disk_action(Box::new(|sb: SuperBlock| {
                let reads = sb
                    .get_inode_table_block_range()
                    .into_iter()
                    .map(|i| Block::get_block(i))
                    .collect();
                sequence(reads)
            })),
        );

        map(d, file_system::lift(Box::new(&Inode::blocks_to_inodes)))
    }
    // Get the superblock
    // Read the Blocks that take up the inode table
    // Convert each Block data into a array of Inodes
    // flatten array of inodes
    // get the inode with the number passed in
    pub fn get_inode<'a>(s: u32) -> DiskAction<'a, Option<Inode>> {
        let d = Inode::get_inode_table();
        let d = map(
            d,
            file_system::lift(Box::new(move |vec| vec.into_iter().find(|x| x.number == s))),
        );
        map(d, Box::new(|x| x.flatten()))
    }

    pub fn get_free_inode<'a>() -> DiskAction<'a, Option<Inode>> {
        let d = Inode::get_inode_table();
        let d = map(
            d,
            file_system::lift(Box::new(|v| v.into_iter().find(|x| x.start_block == None))),
        );
        map(d, Box::new(|x| x.flatten()))
    }

    pub fn parse_inodes(s: &str) -> Option<Vec<Inode>> {
        serde_json::from_str(s).ok()
    }

    pub fn blocks_to_inodes(b: Vec<Option<Block>>) -> Vec<Inode> {
        let option_inodes = Inode::remove_options(b)
            .into_iter()
            .map(|x| Inode::parse_inodes(&x.data))
            .collect();
        Inode::remove_options(option_inodes)
            .into_iter()
            .flatten()
            .collect()
    }

    fn remove_options<A>(b: Vec<Option<A>>) -> Vec<A> {
        b.into_iter()
            .fold(Vec::<A>::new(), |mut acc, curr| match curr {
                Some(a) => {
                    acc.push(a);
                    acc
                }
                None => acc,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_inode_should_return_expected() {
        let expected_data = Inode {
            number: 1,
            start_block: Some(5),
        };

        let disk = Disk::new("./file-system/sda1");
        let (data, _) = Inode::get_inode(1)(disk);
        assert_eq!(data, Some(expected_data));
    }

    #[test]
    fn get_free_inode_should_return_expected() {
        let expected_data = Inode {
            number: 2,
            start_block: None,
        };

        let disk = Disk::new("./file-system/sda1");
        let (data, _) = Inode::get_free_inode()(disk);
        assert_eq!(data, Some(expected_data));
    }
}
