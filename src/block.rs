use crate::disk::*;
use crate::file_system;
use crate::line_handler;
use serde::{Deserialize, Serialize};

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
    pub fn get_block<'a>(block_number: u32) -> DiskAction<'a, Option<Block>> {
        let d = line_handler::read(block_number);
        let parse = Box::new(move |s: String| serde_json::from_str::<Block>(&s[..]).ok());
        let d = map(d, file_system::lift(parse));
        map(d, Box::new(|x| x.flatten()))
    }
}

impl SuperBlock {
    // Return the super_block for a disk
    pub fn get_super_block<'a>() -> DiskAction<'a, Option<SuperBlock>> {
        let d = Block::get_block(1);
        let parse = Box::new(move |s: Block| serde_json::from_str::<SuperBlock>(&s.data).ok());
        let d = map(d, file_system::lift(parse));
        map(d, Box::new(|x| x.flatten()))
    }

    // Inode table blocks take up 10% of available blocks
    pub fn get_inode_table_block_range(&self) -> std::ops::Range<u32> {
        (2..(self.total_blocks as f32 * 0.10) as u32 + 2)
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
        let disk = Disk::new("./file-system/sda1");
        let (data, _) = Block::get_block(1)(disk);
        assert_eq!(data, Some(expected_block));
    }

    #[test]
    fn get_superblock_should_return_expected() {
        let expected_superblock = SuperBlock {
            magic_number: "0x70736575646F4653".into(),
            total_blocks: 10,
        };

        let disk = Disk::new("./file-system/sda1");
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
}
