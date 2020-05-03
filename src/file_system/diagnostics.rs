use super::block::*;
use super::disk::*;
use super::inode::*;

#[derive(Copy, PartialEq, Clone, Debug)]
pub struct DiskDiagnostics {
    valid_magic_number: bool,
    total_reads: u32,
    total_writes: u32,
    total_blocks: u32,
    total_inodes: u32,
    total_free_inodes: u32,
    total_free_blocks: u32,
}

impl DiskDiagnostics {
    pub fn get_diagnostics<'a>() -> DiskAction<'a, Option<DiskDiagnostics>> {
        let sb = SuperBlock::get_super_block();
        let free_blocks = Block::get_all_free_data_blocks();
        let free_inodes = Inode::get_free_inodes();

        let d = map2(sb, free_blocks, Box::new(|a, b| (a, b)));
        let d = map2(d, free_inodes, Box::new(|(a, b), c| (a, b, c)));

        Box::new(move |disk| {
            let (t, disk2) = d(disk);
            println!("{:?}", t);
            let (sb, blocks, inodes) = t;
            let res = sb.map(|x| DiskDiagnostics {
                valid_magic_number: x.valid_super_block(),
                total_reads: disk2.reads,
                total_writes: disk2.writes,
                total_blocks: x.total_blocks,
                total_inodes: x.get_inode_count(),
                total_free_inodes: inodes.len() as u32,
                total_free_blocks: blocks.len() as u32,
            });
            (res, disk2)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_diagnostics_should_return_expected() {
        let expected = DiskDiagnostics {
            valid_magic_number: true,
            total_reads: 12,
            total_writes: 0,
            total_blocks: 10,
            total_inodes: 10,
            total_free_inodes: 1,
            total_free_blocks: 3,
        };

        let disk = Disk::new("./test-files/sda1");
        let (res, _) = DiskDiagnostics::get_diagnostics()(disk);
        assert_eq!(res, Some(expected));
    }
}
