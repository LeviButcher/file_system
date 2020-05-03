use crate::disk::*;
use std::fs;

pub fn read<'a>(line: u32) -> DiskAction<'a, Option<String>> {
    let index_line = line - 1;
    Box::new(move |disk: Disk| {
        let d = disk.read();
        let r = fs::read_to_string(disk.file)
            .ok()
            .and_then(move |l: String| l.lines().map(|x| x.to_owned()).nth(index_line as usize));
        (r, d)
    })
}
// Line [1..]
pub fn write<'a>(line: u32, data: String) -> DiskAction<'a, Option<String>> {
    let indexed_line = line - 1;

    Box::new(move |disk: Disk| {
        let data = data.clone();

        let r = fs::read_to_string(disk.file)
            .ok()
            .map(Box::new(move |s: String| {
                let r: Vec<String> = s.lines().map(|x| x.to_owned()).collect();

                let extra_lines_to_add = if line > r.len() as u32 {
                    line - r.len() as u32
                } else {
                    0
                };
                let extra_lines: Vec<String> = (0..extra_lines_to_add)
                    .into_iter()
                    .map(|_| "".into())
                    .collect();

                r.into_iter()
                    .chain(extra_lines.into_iter())
                    .enumerate()
                    .map(Box::new(move |(i, v)| {
                        if i as u32 == indexed_line {
                            return data.clone();
                        }
                        v
                    }))
                    .collect()
            }))
            .and_then(Box::new(|x: Vec<String>| {
                let file_string: String = x.join("\n");

                fs::write(disk.file, file_string)
                    .ok()
                    .and_then(move |_| x.get(indexed_line as usize).map(|x| x.to_owned()))
            }));
        (r, disk.write())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: these tests should really be cleaning up the changes to the file after running
    // But Rust's tests runner doesn't have a tear down or set up. :/

    #[test]
    fn read_should_return_file_line() {
        let disk = Disk::new("./test-files/line_handler_test_file.txt");
        let (data, updated_disk) = read(2)(disk);
        assert_eq!(data, Some("FP4TheWin".into()));
        assert_eq!(updated_disk.reads, 1);
    }

    #[test]
    fn read_line_does_not_exist_should_return_none() {
        let disk = Disk::new("./test-files/line_handler_test_file.txt");
        let (data, updated_disk) = read(10)(disk);
        assert_eq!(data, None);
        assert_eq!(updated_disk.reads, 1);
    }

    #[test]
    fn read_file_does_not_exist_should_return_none() {
        let disk = Disk::new("rust_ownership_makes_me_cry_sometimes.rs");
        let (data, updated_disk) = read(10)(disk);
        assert_eq!(data, None);
        assert_eq!(updated_disk.reads, 1);
    }

    #[test]
    fn multiple_reads_should_return_expected() {
        let expected_data = vec![
            Some("Nope".into()),
            Some("super_awesome".into()),
            Some("Yeah".into()),
            Some("FP4TheWin".into()),
        ];
        let disk = Disk::new("./test-files/line_handler_test_file.txt");
        let reads = vec![read(3), read(5), read(1), read(2)];
        let mega_read = sequence(reads);
        let (data, updated_disk) = mega_read(disk);
        assert_eq!(updated_disk.reads, 4);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn write_should_return_expected() {
        let expected_data = Some("super_awesome".into());
        let disk = Disk::new("./test-files/line_handler_test_file.txt");
        let (data, updated_disk) = write(5, "super_awesome".into())(disk);
        assert_eq!(data, expected_data);
        assert_eq!(updated_disk.writes, 1);
        assert_eq!(updated_disk.reads, 0);
        let (data, updated_disk) = read(5)(updated_disk);
        assert_eq!(data, expected_data);
        assert_eq!(updated_disk.writes, 1);
        assert_eq!(updated_disk.reads, 1);
    }

    #[test]
    fn write_line_exists_should_return_expected() {
        let disk = Disk::new("./test-files/line_handler_write_test.txt");
        let (_, updated_disk) = write(1, "Yeah".into())(disk);
        assert_eq!(updated_disk.writes, 1);
    }
}
