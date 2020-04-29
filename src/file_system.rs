// Print Directory
// Cat out a File
// Save A new file
// Delete a File
// Print Diagnostics -> Disk Responsibility

use crate::block::*;
use crate::directory::*;
use crate::disk::*;
use crate::inode::*;
use crate::line_handler;

fn write_file_name<'a>(i: u32, file_name: &str) -> DiskAction<'a, Option<u32>> {
    unit(None)
}

fn get_data_blocks<'a>(s: Inode) -> DiskAction<'a, Vec<Block>> {
    Box::new(move |d: Disk| (vec![], d))
}

fn get_free_data_blocks<'a>(num: usize) -> DiskAction<'a, Vec<Block>> {
    Box::new(move |d: Disk| (vec![], d))
}

// List of Blocks
fn to_data(a: Vec<Block>) -> &'static str {
    ""
}

pub fn lift<'a, A: 'a, B: 'a>(f: Box<dyn Fn(A) -> B>) -> Box<dyn Fn(Option<A>) -> Option<B> + 'a> {
    Box::new(move |x| match x {
        Some(a) => Some(f(a)),
        _ => None,
    })
}

pub fn lift_disk_action<'a, A: 'a, B: 'a>(
    f: Box<dyn Fn(A) -> DiskAction<'a, B>>,
) -> Box<dyn Fn(Option<A>) -> DiskAction<'a, Option<B>> + 'a> {
    Box::new(move |a: Option<A>| match a {
        Some(b) => map(f(b), Box::new(|c| Some(c))),
        None => Box::new(|disk| (None, disk)),
    })
}

fn set_inode_blocks(a: Option<Inode>, b: Option<Vec<Block>>) -> Option<(Inode, Vec<Block>)> {
    None
}

fn write_inode_and_blocks<'a>(
    a: (Inode, Vec<Block>),
) -> DiskAction<'a, Option<(Inode, Vec<Block>)>> {
    Box::new(move |d: Disk| (None, d))
}

fn set_data_blocks_data<'a, A>(d: (Vec<Block>, &Vec<A>)) -> Option<Vec<Block>> {
    None
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
    fn read_file<'a>(file_name: String) -> DiskAction<'a, Option<&'a str>> {
        let d = Directory::get_directory();
        let d = map(
            d,
            lift(Box::new(move |dir: Directory| dir.find(&file_name[..]))),
        );
        let d = map(d, Box::new(&|x: Option<Option<u32>>| x.flatten()));
        let d = flat_map(d, lift_disk_action(Box::new(Inode::get_inode)));
        let d = map(d, Box::new(|a| a.flatten()));
        let d = flat_map(d, lift_disk_action(Box::new(get_data_blocks)));
        map(d, lift(Box::new(to_data)))
    }

    /// Give a disk we will
    /// 1. Chunk data into 1024 char array
    /// 1. Get Free Inode
    /// 2. Get Amount of Free Blocks as char array
    /// 3. Set Free Inodes startBlock to first block in char array and follow through the blocks till end
    /// 4. Write Free Inode and Start Blocks to disk
    /// 5. Write File Name and Inode Number to directory
    fn save_as_file<'a>(file_name: String, data: &str) -> DiskAction<'a, Option<u32>> {
        // TODO: check file doesn't exist first
        let data = data
            .split_whitespace() // TODO: Chunk this 1024
            .map(|x| x.into())
            .collect::<Vec<String>>(); // Set up data for storage
        let d = Inode::get_free_inode(); // Get A Free Inode
        let data_block = get_free_data_blocks(data.len()); // Get Enough Free Blocks for data
        let data_block = map(
            data_block,
            Box::new(move |x| set_data_blocks_data((x, &data))),
        ); // Set Data On Free Blocks
        let d = map2(d, data_block, Box::new(set_inode_blocks)); // Combine inode and data_blocks and set them up
        let d = flat_map(d, lift_disk_action(Box::new(write_inode_and_blocks))); // Write out the inode and data blocks
        let d = map(d, Box::new(|x| x.flatten()));
        flat_map(
            d,
            Box::new(move |a| match a {
                Some((i, _)) => write_file_name(i.number, &file_name),
                None => unit(None),
            }),
        ) // Write out the file name to the directory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk::Disk;

    // #[test]
    // fn read_file_should_return_expected() {
    //     let disk = Disk::new("./file-system/sda1");
    //     let (data, _) = FileSystem::read_file("secret.txt".into())(disk);
    //     assert_eq!(data, Some("Super Secret"));
    // }
}
