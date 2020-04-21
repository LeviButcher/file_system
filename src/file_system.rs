use crate::utils::*;
use std::fs::File;

trait FileSystem {
    fn diagnostics(d: Disk);
    fn create(f: File) -> bool;
    fn format(f: File) -> bool;
    fn mount(f: File) -> bool;
    fn un_mount() -> bool;
    fn read_inode(a: u16) -> Inode;
    fn write_inode(a: u16, i: Inode) -> bool;
    fn get_inode(a: u16) -> Inode;
    fn read_block(a: u16) -> Block;
    fn write_block(a: u16, block: Block) -> bool;
    fn getFreeBlock() -> Block;
}
