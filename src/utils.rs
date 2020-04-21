use std::fs::File;

pub struct Disk {
    file: File,
    blocks: u32,
    reads: f32,
    writes: f32,
    mounted: bool,
}
pub enum InodeType {
    Free, File, Directory, SymLink
}

pub struct Inode {
    number: u32,
    type: InodeType,
    startBlock: u32
    size: u32,
    // Change to time datatype
    cTime: u32,


}
pub struct Block {}
pub struct SuperBlock {}

pub struct SuperBlock {
    // should always equal = 0x70736575646F4653
    magicNumber: u32,
    // Number of lines in system
    totalBlocks: u32,
    freeBlocks: Vec<u32>,
    totalInodes: u32,
    freeInodes: Vec<u32>,
}
