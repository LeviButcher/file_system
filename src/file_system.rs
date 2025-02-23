// Print Directory
// Cat out a File
// Save A new file
// Delete a File
// Print Diagnostics -> Disk Responsibility
// Create a new Disk
// Format a Disk

mod block;
mod diagnostics;
mod directory;
pub mod disk;
mod inode;
mod line_handler;
mod utils;

use block::*;
use diagnostics::*;
use directory::*;
pub use disk::*;
use inode::*;

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

pub fn get_file_inode_and_blocks<'a>(
    file_name: String,
) -> DiskAction<'a, Option<(Inode, Vec<Block>)>> {
    let d = Directory::get_directory();
    let d = map(
        d,
        utils::lift(Box::new(move |dir: Directory| dir.find(&file_name[..]))),
    );
    let d = map(d, Box::new(&|x: Option<Option<u32>>| x.flatten()));
    let d = flat_map(d, utils::lift_disk_action(Box::new(Inode::get_inode)));
    let d = flatten_option(d);
    let d = flat_map(
        d,
        utils::lift_disk_action(Box::new(Inode::get_inode_blocks)),
    );
    flatten_option(d)
}

#[derive(Debug)]
pub struct FileSystem {}
impl FileSystem {
    pub fn read_file<'a>(file_name: String) -> DiskAction<'a, Option<String>> {
        let d = get_file_inode_and_blocks(file_name);
        map(d, utils::lift(Box::new(|(_, b)| Block::blocks_to_data(b))))
    }

    pub fn save_as_file<'a>(file_name: String, data: String) -> DiskAction<'a, Option<u32>> {
        let data = utils::string_to_block_data_chunks(data);
        let d = Inode::get_free_inode(); // Get A Free Inode
        let data_block = Block::get_free_data_blocks(data.len()); // Get Enough Free Blocks for data
        let data_block = map(
            data_block,
            Box::new(move |x| Block::set_data_blocks_data((x, data.clone()))),
        ); // Set Data On Free Blocks
        let d = map2(d, data_block, Box::new(Inode::set_inode_blocks)); // Combine inode and data_blocks and set them up
        let d = flat_map(d, utils::lift_disk_action(Box::new(write_inode_and_blocks))); // Write out the inode and data blocks
        let d = flatten_option(d);
        flat_map(
            d,
            Box::new(move |a| match a {
                Some((i, _)) => Directory::write_file_name(i.number, file_name.clone()),
                None => unit(None),
            }),
        ) // Write out the file name to the directory
    }

    // ls the directory
    pub fn get_directory<'a>() -> DiskAction<'a, Option<Directory>> {
        Directory::get_directory()
    }

    // Size == how many line
    pub fn create_disk<'a>(file: String, size: u32) -> bool {
        use std::fs;

        fs::File::create(file.clone())
            .ok()
            .map(move |_| FileSystem::format(file, size))
            .unwrap_or(false)
    }

    pub fn remove_file<'a>(file_name: String) -> DiskAction<'a, bool> {
        if file_name == "/" || file_name == "." {
            return unit(false);
        }

        let write_inode_and_blocks = get_file_inode_and_blocks(file_name.clone());
        let write_inode_and_blocks = flat_map(
            write_inode_and_blocks,
            utils::lift_disk_action(Box::new(|(i, b)| {
                let free_inode = Inode::free_inode(i);
                let free_blocks = Block::free_blocks(b);
                map2(free_inode, free_blocks, Box::new(|a, b| (a, b)))
            })),
        );
        let write_directory = Directory::remove_file_name(file_name.clone());
        let d = map2(write_inode_and_blocks, write_directory, Box::new(|_, b| b));
        map(d, Box::new(|x| x.is_some()))
    }

    pub fn format<'a>(file_name: String, size: u32) -> bool {
        let disk = Disk::new(&file_name);
        let super_block = SuperBlock::new(size);
        let write_blocks: Vec<DiskAction<Option<Block>>> = (1..size + 1)
            .into_iter()
            .map(|x| {
                if x == 1 {
                    return Block {
                        number: x,
                        b_type: BlockType::Free,
                        data: serde_json::to_string(&super_block).unwrap_or("".into()),
                    };
                } else {
                    Block {
                        number: x,
                        b_type: BlockType::Free,
                        data: "".into(),
                    }
                }
            })
            .map(Block::write_block)
            .collect();

        let write_blocks = sequence(write_blocks);

        let inodes = Inode::generate_inodes(size);
        let write_inodes = Inode::replace_all_inodes(inodes);

        let directory = Directory::default();
        let write_directory = Directory::save_directory(directory);

        let d = map2(write_blocks, write_inodes, Box::new(|a, _| a));
        let d = map2(d, write_directory, Box::new(|_, b| b));
        let (res, _d) = d(disk);
        res.is_some()
    }

    // Check that superblock is valid, if so return disk
    // POTENTIAL PROBLEM, file_name needs to last as long as disk, so
    // in the shell it may be a issue if the users command is dropped
    // I hope not :/
    pub fn mount<'a>(file_name: &'a str) -> Option<Disk<'a>> {
        let disk = Disk::new(&file_name);
        let (res, disk) = SuperBlock::get_super_block()(disk);
        res.and_then(|s| {
            if s.valid_super_block() {
                Some(disk)
            } else {
                None
            }
        })
    }

    pub fn get_diagnostic<'a>() -> DiskAction<'a, Option<DiskDiagnostics>> {
        DiskDiagnostics::get_diagnostics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_file_should_return_expected() {
        let disk = Disk::new("./test-files/sda1");
        let (data, _) = FileSystem::read_file("secret.txt".into())(disk);
        assert_eq!(data, Some("SomebodyOnceToldMe".into()));
    }

    #[test]
    fn write_file_should_return_expected() {
        use std::fs;

        let file_data = fs::read_to_string("./test-files/sda1").unwrap_or("".into());
        fs::write("./test-files/sda1_write_file_test", file_data).unwrap();

        let disk = Disk::new("./test-files/sda1_write_file_test");
        let file_data: String = "Ten long years have I waited for the day that COBOL will come back to rise from the bits".into();

        let (result, disk) =
            FileSystem::save_as_file("cobol_rise.txt".into(), file_data.clone())(disk);
        assert_eq!(result, Some(2));

        let (data, _) = FileSystem::read_file("cobol_rise.txt".into())(disk);
        assert_eq!(data, Some(file_data));
    }

    #[test]
    fn format_should_return_expected() {
        let file: String = "./test-files/format_test".into();
        let blocks = 50;
        let res = FileSystem::format(file.clone(), blocks);
        assert!(res);
        let disk = Disk::new(&file);
        let (sb, _) = SuperBlock::get_super_block()(disk);
        assert_eq!(sb.unwrap().total_blocks, blocks);
    }

    #[test]
    fn create_disk_should_return_expected() {
        let file: String = "./test-files/create_test".into();
        let blocks = 50;
        let res = FileSystem::create_disk(file.clone(), blocks);
        assert!(res);
        let disk = Disk::new(&file);
        let (sb, _) = SuperBlock::get_super_block()(disk);
        assert_eq!(sb.unwrap().total_blocks, blocks);
    }

    #[test]
    fn mount_should_mount_successfully() {
        let res = FileSystem::mount("./test-files/sda1".into());
        assert!(res.is_some());
    }

    #[test]
    fn mount_should_mount_unsuccessfully() {
        let res = FileSystem::mount("./test-files/line_handler_test_file.txt".into());
        assert!(res.is_none());
    }

    #[test]
    fn remove_file_should_return_expected() {
        use std::fs;

        let file_data = fs::read_to_string("./test-files/sda1").unwrap_or("".into());
        fs::write("./test-files/remove_file_test", file_data).unwrap();
        let disk = Disk::new("./test-files/remove_file_test");

        let (result, disk) = FileSystem::remove_file("secret.txt".into())(disk);
        assert_eq!(result, true);

        let (data, _) = FileSystem::read_file("secret.txt".into())(disk);
        assert_eq!(data, None);
    }
}
