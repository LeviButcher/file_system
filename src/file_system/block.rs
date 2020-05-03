use super::disk::*;
use super::line_handler;
use super::utils;
use serde::{Deserialize, Serialize};

static MAGIC_NUMBER: &str = "0x70736575646F4653";

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Block {
    pub number: u32,
    pub b_type: BlockType,
    pub data: String,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum BlockType {
    Free,      // Free to use Block
    Next(u32), // Next block in chain
    End,       // End of chain of blocks
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct SuperBlock {
    pub magic_number: String,
    pub total_blocks: u32,
}

impl Block {
    pub fn free(self) -> Block {
        Block {
            number: self.number,
            b_type: BlockType::Free,
            data: String::new(),
        }
    }

    pub fn get_block<'a>(block_number: u32) -> DiskAction<'a, Option<Block>> {
        let d = line_handler::read(block_number);
        let parse = Box::new(move |s: String| serde_json::from_str::<Block>(&s[..]).ok());
        let d = map(d, utils::lift(parse));
        map(d, Box::new(|x| x.flatten()))
    }

    pub fn blocks_to_data(a: Vec<Block>) -> String {
        a.into_iter().fold("".into(), |acc, curr| acc + &curr.data)
    }

    pub fn get_all_blocks<'a>() -> DiskAction<'a, Vec<Block>> {
        let d = SuperBlock::get_super_block();

        let d = map(d, utils::lift(Box::new(|x| x.get_storage_block_range())));
        let d = flat_map(
            d,
            utils::lift_disk_action(Box::new(|storage_range| {
                let reads = storage_range
                    .into_iter()
                    .map(|x| Block::get_block(x))
                    .collect();
                sequence(reads)
            })),
        );
        let d = map(d, utils::lift(Box::new(utils::remove_options)));
        map(d, Box::new(|x| x.unwrap_or(vec![])))
    }

    pub fn get_all_free_data_blocks<'a>() -> DiskAction<'a, Vec<Block>> {
        let d = Block::get_all_blocks();
        map(
            d,
            Box::new(move |x| {
                x.into_iter()
                    .filter(|a| a.b_type == BlockType::Free)
                    .collect()
            }),
        )
    }

    pub fn get_free_data_blocks<'a>(num: usize) -> DiskAction<'a, Vec<Block>> {
        let d = Block::get_all_free_data_blocks();
        map(d, Box::new(move |x| x.into_iter().take(num).collect()))
    }

    pub fn set_data_blocks_data(d: (Vec<Block>, Vec<String>)) -> Vec<Block> {
        let (blocks, data) = d;
        blocks
            .into_iter()
            .zip(data.into_iter())
            .map(|(mut block, datum)| {
                block.data = datum;
                block
            })
            .collect()
    }

    pub fn write_block<'a>(b: Block) -> DiskAction<'a, Option<Block>> {
        let d = line_handler::write(
            b.number,
            serde_json::to_string(&b).ok().unwrap_or("".into()),
        );
        let d = map(d, utils::lift(Box::new(|x| serde_json::from_str(&x).ok())));
        map(d, Box::new(|x| x.flatten()))
    }

    pub fn free_blocks<'a>(b: Vec<Block>) -> DiskAction<'a, Vec<Block>> {
        let d = b
            .into_iter()
            .map(|x| x.free())
            .map(|x| Block::write_block(x))
            .collect();
        let d = sequence(d);

        map(d, Box::new(utils::remove_options))
    }
}

impl SuperBlock {
    pub fn valid_super_block(&self) -> bool {
        self.magic_number == MAGIC_NUMBER
    }

    // Return the super_block for a disk
    pub fn get_super_block<'a>() -> DiskAction<'a, Option<SuperBlock>> {
        let d = Block::get_block(1);
        let parse = Box::new(move |s: Block| serde_json::from_str::<SuperBlock>(&s.data).ok());
        let d = map(d, utils::lift(parse));
        map(d, Box::new(|x| x.flatten()))
    }

    // Inode table blocks take up 10% of available blocks
    pub fn get_inode_table_block_range(&self) -> std::ops::Range<u32> {
        (2..(self.total_blocks as f32 * 0.10) as u32 + 2)
    }

    pub fn get_storage_block_range(&self) -> std::ops::Range<u32> {
        let inodes_end = (self.total_blocks as f32 * 0.10) as u32 + 2;
        (inodes_end..self.total_blocks + 1)
    }

    pub fn get_inode_count(&self) -> u32 {
        let inode_table_blocks = (self.total_blocks as f32 * 0.10) as u32;
        inode_table_blocks * self.total_blocks
    }

    pub fn new(size: u32) -> SuperBlock {
        SuperBlock {
            magic_number: MAGIC_NUMBER.to_owned(),
            total_blocks: size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_block_should_return_expected() {
        let expected_block = Block {
            number: 1,
            data: "{\"magic_number\":\"0x70736575646F4653\",\"total_blocks\":10}".into(),
            b_type: BlockType::End,
        };
        let disk = Disk::new("./test-files/sda1");
        let (data, disk) = Block::get_block(1)(disk);
        assert_eq!(data, Some(expected_block));
        assert_eq!(disk.reads, 1);
    }

    #[test]
    fn get_superblock_should_return_expected() {
        let expected_superblock = SuperBlock {
            magic_number: "0x70736575646F4653".into(),
            total_blocks: 10,
        };

        let disk = Disk::new("./test-files/sda1");
        let (data, _) = SuperBlock::get_super_block()(disk);
        assert_eq!(data, Some(expected_superblock));
    }

    #[test]
    fn get_inode_table_range_should_return_expected() {
        // 10 total blocks should give range 1-1
        let expected = 2..3;
        let s = SuperBlock {
            magic_number: "".into(),
            total_blocks: 10,
        };
        assert_eq!(s.get_inode_table_block_range(), expected);
    }

    #[test]
    fn get_free_data_blocks_should_return_amount_expected_and_be_free() {
        let expected_count = 2;
        let disk = Disk::new("./test-files/sda1");
        let (data, _) = Block::get_free_data_blocks(expected_count)(disk);
        assert_eq!(data.len(), expected_count);
        let all_free = data.into_iter().all(|x| x.b_type == BlockType::Free);
        assert_eq!(all_free, true);
    }

    #[test]
    fn get_storage_block_range() {
        let expected = 3..11;
        let sb = SuperBlock {
            magic_number: "asdfas".into(),
            total_blocks: 10,
        };
        assert_eq!(sb.get_storage_block_range(), expected);
    }

    #[test]
    fn set_data_block_data_should_return_expected() {
        let expected_blocks = vec![
            Block {
                number: 1,
                data: "Somebody".into(),
                b_type: BlockType::Free,
            },
            Block {
                number: 2,
                data: "Once".into(),
                b_type: BlockType::Free,
            },
            Block {
                number: 3,
                data: "Told".into(),
                b_type: BlockType::Free,
            },
        ];
        let blocks = vec![
            Block {
                number: 1,
                data: "".into(),
                b_type: BlockType::Free,
            },
            Block {
                number: 2,
                data: "".into(),
                b_type: BlockType::Free,
            },
            Block {
                number: 3,
                data: "".into(),
                b_type: BlockType::Free,
            },
        ];
        let data = vec!["Somebody".into(), "Once".into(), "Told".into()];

        let res = Block::set_data_blocks_data((blocks, data));
        assert_eq!(expected_blocks, res);
    }
}
