use crate::utils::*;
use std::fs::File;

trait DiskEmulator {
    fn open(f: File) -> bool;
    fn close(d: Disk) -> bool;
    fn read(d: Disk, blockId: u16) -> Block;
    fn write(d: Disk, blockId: u16, b: Block) -> bool;
}
