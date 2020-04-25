use crate::Disk::*;
// Describe or writing lines of a file
// Eventually use to read or write a line to a file
enum LineHandler {
    Read(u32),
    Write(u32),
}

pub fn read(line: u32) -> DiskAction<Option<&'static str>> {
    unit(Some(""))
}

pub fn write<A>(line: u32, val: A) -> DiskAction<bool> {
    unit(true)
}

#[cfg(test)]
mod tests {
    #[test]
    fn name() {
        unimplemented!();
    }
}
