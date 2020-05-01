// Print Directory
// Cat out a File
// Save A new file
// Delete a File
// Print Diagnostics -> Disk Responsibility

use crate::block::*;
use crate::directory::*;
use crate::disk::*;
use crate::inode::*;
use crate::utils;

pub fn write_inode_and_blocks<'a>(
    a: (Inode, Vec<Block>),
) -> DiskAction<'a, Option<(Inode, Vec<Block>)>> {
    let (i, blocks) = a;
    let write_inode = Inode::write_inode(i);
    let write_blocks = blocks.into_iter().map(|b| Block::write_block(b)).collect();
    let write_blocks = sequence(write_blocks);
    let write_blocks = map(write_blocks, Box::new(|w| utils::remove_options(w)));

    map2(
        write_inode,
        write_blocks,
        Box::new(|a, b| a.map(|aa| (aa, b))),
    )
}

struct FileSystem {}
impl FileSystem {
    /// Given a Disk we will
    /// 1. Read Inode 1
    /// 2. Construct Data of Inode 1
    /// 3. Get Data Blocks of Inode
    /// 4. Parse that into a Directory
    /// 5. Search the Directory map for key of file_name
    /// 6. If found, take associated inode number and construct Inode Data
    /// 7. Return Inode Data
    fn read_file<'a>(file_name: String) -> DiskAction<'a, Option<String>> {
        let d = Directory::get_directory();
        let d = map(
            d,
            utils::lift(Box::new(move |dir: Directory| dir.find(&file_name[..]))),
        );
        let d = map(d, Box::new(&|x: Option<Option<u32>>| x.flatten()));
        let d = flat_map(d, utils::lift_disk_action(Box::new(Inode::get_inode)));
        let d = map(d, Box::new(|a| a.flatten()));
        let d = flat_map(
            d,
            utils::lift_disk_action(Box::new(Inode::get_inode_blocks)),
        );
        let d = map(d, Box::new(|a| a.flatten()));
        map(d, utils::lift(Box::new(Block::blocks_to_data)))
    }

    /// Give a disk we will
    /// 1. Chunk data into 1024 char array
    /// 1. Get Free Inode
    /// 2. Get Amount of Free Blocks as char array
    /// 3. Set Free Inodes startBlock to first block in char array and follow through the blocks till end
    /// 4. Write Free Inode and Start Blocks to disk
    /// 5. Write File Name and Inode Number to directory
    ///
    /// Return back inode number
    fn save_as_file<'a>(file_name: String, data: String) -> DiskAction<'a, Option<u32>> {
        let data = utils::string_to_block_data_chunks(data);
        let d = Inode::get_free_inode(); // Get A Free Inode
        let data_block = Block::get_free_data_blocks(data.len()); // Get Enough Free Blocks for data
        let data_block = map(
            data_block,
            Box::new(move |x| Block::set_data_blocks_data((x, data.clone()))),
        ); // Set Data On Free Blocks
        let d = map2(d, data_block, Box::new(Inode::set_inode_blocks)); // Combine inode and data_blocks and set them up
        let d = flat_map(d, utils::lift_disk_action(Box::new(write_inode_and_blocks))); // Write out the inode and data blocks
        let d = map(d, Box::new(|x| x.flatten()));
        flat_map(
            d,
            Box::new(move |a| match a {
                Some((i, _)) => Directory::write_file_name(i.number, file_name.clone()),
                None => unit(None),
            }),
        ) // Write out the file name to the directory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk::Disk;

    #[test]
    fn read_file_should_return_expected() {
        let disk = Disk::new("./test-files/sda1");
        let (data, _) = FileSystem::read_file("secret.txt".into())(disk);
        assert_eq!(data, Some("SomebodyOnceToldMe".into()));
    }

    #[test]
    fn write_file_should_return_expected() {
        use std::fs::File;
        use std::io::Read;
        use std::io::Write;

        let mut f = File::open("./test-files/sda1").unwrap();
        let mut file_data = String::new();
        f.read_to_string(&mut file_data);
        f.flush();

        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .open("./test-files/sda1_write_file_test")
            .unwrap();

        f.write(file_data.as_bytes()).unwrap();
        f.flush();

        let disk = Disk::new("./test-files/sda1_write_file_test");
        let file_data: String = "Ten long years have I waited for the day that COBOL will come back to rise from the bits".into();
        let (result, _) = FileSystem::save_as_file("cobol_rise.txt".into(), file_data)(disk);
        assert_eq!(result, Some(2));
    }
}
