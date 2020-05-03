use crate::block::*;
use crate::disk::*;
use crate::utils;
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
    pub fn generate_inodes(size: u32) -> Vec<Inode> {
        let inode_table_blocks = (size as f32 * 0.10) as u32;
        let total_inodes = inode_table_blocks * size;
        (1..total_inodes + 1)
            .into_iter()
            .map(|x| Inode {
                number: x,
                start_block: None,
            })
            .collect()
    }

    fn get_inode_table<'a>() -> DiskAction<'a, Option<Vec<Inode>>> {
        let d = SuperBlock::get_super_block();
        let d = flat_map(
            d,
            utils::lift_disk_action(Box::new(|sb: SuperBlock| {
                let reads = sb
                    .get_inode_table_block_range()
                    .into_iter()
                    .map(|i| Block::get_block(i))
                    .collect();
                sequence(reads)
            })),
        );

        map(d, utils::lift(Box::new(&Inode::blocks_to_inodes)))
    }

    pub fn get_inode<'a>(s: u32) -> DiskAction<'a, Option<Inode>> {
        let d = Inode::get_inode_table();
        let d = map(
            d,
            utils::lift(Box::new(move |vec| vec.into_iter().find(|x| x.number == s))),
        );
        map(d, Box::new(|x| x.flatten()))
    }

    pub fn get_free_inode<'a>() -> DiskAction<'a, Option<Inode>> {
        let d = Inode::get_inode_table();
        let d = map(
            d,
            utils::lift(Box::new(|v| v.into_iter().find(|x| x.start_block == None))),
        );
        map(d, Box::new(|x| x.flatten()))
    }

    pub fn parse_inodes(s: &str) -> Option<Vec<Inode>> {
        serde_json::from_str(s).ok()
    }

    pub fn blocks_to_inodes(b: Vec<Option<Block>>) -> Vec<Inode> {
        let option_inodes = utils::remove_options(b)
            .into_iter()
            .map(|x| Inode::parse_inodes(&x.data))
            .collect();
        utils::remove_options(option_inodes)
            .into_iter()
            .flatten()
            .collect()
    }

    // Given a Inode, return all link list of blocks
    pub fn get_inode_blocks<'a>(i: Inode) -> DiskAction<'a, Option<(Inode, Vec<Block>)>> {
        // This could be improved with unfold, :/
        // Or maybe map2?
        Box::new(move |mut disk| {
            let r = i.start_block;
            let r = r.map(|start| Block::get_block(start));
            let blocks = r.map(|read_block| {
                let mut blocks = Vec::<Block>::new();
                let (data, mut disk2) = read_block(disk);
                data.map(|mut x| {
                    blocks.push(x.clone());
                    while x.clone().b_type != BlockType::End {
                        if let BlockType::Next(num) = x.clone().b_type {
                            let (data, disk3) = Block::get_block(num)(disk2);
                            disk2 = disk3;
                            x = data.unwrap();
                            blocks.push(x.clone());
                        }
                    }
                });
                disk = disk2;
                blocks
            });
            (blocks.map(|b| (i, b)), disk)
        })
    }

    pub fn set_inode_blocks(a: Option<Inode>, mut b: Vec<Block>) -> Option<(Inode, Vec<Block>)> {
        a.and_then(|mut i| {
            b.split_first_mut().map(move |(first, rest)| {
                i.start_block = Some(first.number);

                let new_blocks: Vec<Block> = vec![first.clone()]
                    .into_iter()
                    .chain(rest.to_owned())
                    .collect();

                let new_blocks = new_blocks
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(i, mut v)| {
                        let next = new_blocks.get(i + 1);
                        if let Some(next_block) = next {
                            v.b_type = BlockType::Next(next_block.number);
                        } else {
                            v.b_type = BlockType::End
                        }
                        v
                    })
                    .collect();

                (i, new_blocks)
            })
        })
    }

    // inode_table should never be more then INODE_TABLE_SIZE & available Inode Blocks
    pub fn replace_all_inodes<'a>(inode_table: Vec<Inode>) -> DiskAction<'a, Option<Vec<Inode>>> {
        // Read Inode_Table Blocks
        // Set Data of Blocks to table
        // save all blocks
        let r = inode_table
            .chunks(INODE_TABLE_SIZE as usize)
            .map(|x| x.iter().map(|i| *i).collect::<Vec<Inode>>())
            .map(|x| serde_json::to_string(&x).ok().unwrap_or("".into()))
            .collect::<Vec<String>>();

        let d = SuperBlock::get_super_block();
        let d = map(
            d,
            utils::lift(Box::new(|x| x.get_inode_table_block_range())),
        );
        // Read blocks
        let d = flat_map(
            d,
            utils::lift_disk_action(Box::new(|x| {
                let reads = x.into_iter().map(|a| Block::get_block(a)).collect();
                sequence(reads)
            })),
        );
        let d = map(d, utils::lift(Box::new(utils::remove_options)));
        // We have blocks, now set the data of the blocks
        let d = map(
            d,
            utils::lift(Box::new(move |x| {
                Block::set_data_blocks_data((x, r.clone()))
            })),
        );
        // Blocks have been set lets write them
        let d = flat_map(
            d,
            utils::lift_disk_action(Box::new(|x| {
                let writes = x
                    .into_iter()
                    .map(|mut b| {
                        b.b_type = BlockType::End;
                        b
                    })
                    .map(|b| Block::write_block(b))
                    .collect();
                sequence(writes)
            })),
        );
        let d = map(d, utils::lift(Box::new(utils::remove_options)));
        map(
            d,
            utils::lift(Box::new(|blocks| -> Vec<Inode> {
                let i = blocks
                    .into_iter()
                    .map(|b| serde_json::from_str::<Vec<Inode>>(&b.data).ok())
                    .collect();
                let i = utils::remove_options(i);
                i.into_iter().flat_map(|b| b).collect::<Vec<Inode>>()
            })),
        )
    }

    // Read Inode table
    // Replace associated inode with inode
    // Write Inode table back out to blocks
    pub fn write_inode<'a>(i: Inode) -> DiskAction<'a, Option<Inode>> {
        let d = Inode::get_inode_table();
        let d = map(
            d,
            utils::lift(Box::new(move |inodes| {
                inodes
                    .into_iter()
                    .map(|x| {
                        if x.number == i.number {
                            return i;
                        }
                        x
                    })
                    .collect()
            })),
        );
        let d = flat_map(
            d,
            utils::lift_disk_action(Box::new(Inode::replace_all_inodes)),
        );
        let d = map(d, Box::new(|x| x.flatten()));
        let d = map(
            d,
            utils::lift(Box::new(move |inodes| {
                inodes.into_iter().find(|x| x.number == i.number)
            })),
        );
        map(d, Box::new(|x| x.flatten()))
    }

    pub fn free_inode<'a>(mut i: Inode) -> DiskAction<'a, Option<Inode>> {
        i.start_block = None;
        Inode::write_inode(i)
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

        let disk = Disk::new("./test-files/sda1");
        let (data, _) = Inode::get_inode(1)(disk);
        assert_eq!(data, Some(expected_data));
    }

    #[test]
    fn get_free_inode_should_return_expected() {
        let expected_data = Inode {
            number: 2,
            start_block: None,
        };

        let disk = Disk::new("./test-files/sda1");
        let (data, _) = Inode::get_free_inode()(disk);
        assert_eq!(data, Some(expected_data));
    }

    #[test]
    fn get_inode_blocks_should_return_expected() {
        let expected_data = vec![
            Block {
                number: 4,
                b_type: BlockType::Next(6),
                data: "Somebody".into(),
            },
            Block {
                number: 6,
                b_type: BlockType::Next(8),
                data: "Once".into(),
            },
            Block {
                number: 8,
                b_type: BlockType::Next(9),
                data: "Told".into(),
            },
            Block {
                number: 9,
                b_type: BlockType::End,
                data: "Me".into(),
            },
        ];
        let disk = Disk::new("./test-files/sda1");
        let inode = Inode {
            number: 3,
            start_block: Some(4),
        };
        let (data, disk) = Inode::get_inode_blocks(inode)(disk);
        assert_eq!(data, Some((inode, expected_data)));
        assert_eq!(disk.reads, 4);
    }

    #[test]
    fn set_inodes_blocks_should_return_expected() {
        let i = Inode {
            number: 1,
            start_block: None,
        };
        let blocks = vec![
            Block {
                number: 4,
                b_type: BlockType::Free,
                data: "Somebody".into(),
            },
            Block {
                number: 6,
                b_type: BlockType::Free,
                data: "Once".into(),
            },
        ];
        let expected_inode = Inode {
            number: 1,
            start_block: Some(4),
        };
        let expected_blocks = vec![
            Block {
                number: 4,
                b_type: BlockType::Next(6),
                data: "Somebody".into(),
            },
            Block {
                number: 6,
                b_type: BlockType::End,
                data: "Once".into(),
            },
        ];
        let (i, b) = Inode::set_inode_blocks(Some(i), blocks).unwrap();
        assert_eq!(i, expected_inode);
        assert_eq!(b, expected_blocks);
    }

    #[test]
    fn replace_all_inodes_should_return_expected() {
        let inodes = vec![
            Inode {
                number: 1,
                start_block: Some(5),
            },
            Inode {
                number: 2,
                start_block: None,
            },
            Inode {
                number: 3,
                start_block: Some(4),
            },
            Inode {
                number: 4,
                start_block: None,
            },
            Inode {
                number: 5,
                start_block: None,
            },
            Inode {
                number: 6,
                start_block: None,
            },
            Inode {
                number: 7,
                start_block: None,
            },
            Inode {
                number: 8,
                start_block: None,
            },
        ];
        let disk = Disk::new("./test-files/inode_replace_all_test");
        let (data, _) = Inode::replace_all_inodes(inodes.clone())(disk);

        assert_eq!(data, Some(inodes));
    }

    #[test]
    fn write_inode_should_return_expected() {
        let inode = Inode {
            number: 3,
            start_block: Some(42),
        };
        let disk = Disk::new("./test-files/inode_write_test");
        let (data, _) = Inode::write_inode(inode)(disk);

        assert_eq!(data, Some(inode));
    }
}
