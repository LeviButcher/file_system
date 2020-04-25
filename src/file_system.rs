// Print Directory
// Cat out a File
// Save A new file
// Delete a File
// Print Diagnostics -> Disk Responsibility

use crate::line_handler;
use crate::Disk::*;

#[derive(Copy, Clone)]
struct Inode {}
#[derive(Copy, Clone)]
struct Block {}
#[derive(Copy, Clone)]
struct Directory {}

impl Directory {
    fn find(&self, s: &str) -> Option<u32> {
        Some(0)
    }
}

fn find_inode(s: Option<u32>) -> DiskAction<Option<Inode>> {
    unit(Some(Inode {}))
}

fn parse_inode(s: Option<&str>) -> Option<Inode> {
    Some(Inode {})
}

// Should be List of Blocks -> Search for FP package for Copy-able list
fn get_data_blocks(s: Option<Inode>) -> DiskAction<Option<Block>> {
    unit(Some(Block {}))
}

// List of Blocks
fn to_data(a: Block) -> &'static str {
    ""
}

fn parse_directory(s: &str) -> Directory {
    Directory {}
}

fn lift<A, B>(f: Box<dyn Fn(A) -> B>) -> Box<dyn Fn(Option<A>) -> Option<B>> {
    Box::new(move |x| match x {
        Some(a) => Some(f(a)),
        _ => None,
    })
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
    fn read_file(file_name: String) -> DiskAction<Option<&'static str>> {
        // Possible Inode if parsed Correctly
        // WRONG, Block One will have multiple INODEs in them :(
        let d = map(line_handler::read(1), Box::new(parse_inode));
        let d = flatMap(d, Box::new(get_data_blocks));
        let d = map(d, lift(Box::new(to_data)));
        let d = map(d, lift(Box::new(parse_directory)));
        let d = map(d, lift(Box::new(&move |dir: Directory| dir.find(&file_name[..])))); // We got a file_name
        let d = map(d, Box::new(&|x: Option<Option<u32>>| x.flatten()));
        let d = flatMap(d, Box::new(find_inode));
        let d = flatMap(d, Box::new(get_data_blocks));
        map(d, lift(Box::new(to_data))
    }

    fn save_as_file(file_name: &str, data: &str) -> DiskAction<Option<&'static str>> {
        unit(Some("String::new()"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Disk::Disk;

    #[test]
    fn read_file_should_return_expected() {
        let v = FileSystem::read_file("mywife");

        let d = unit(5);
        let (a, b) = d(Disk { name: "mywife" });
        println!("{}", a);
    }
}
