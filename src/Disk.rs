#[derive(Copy, Debug, Clone)]
pub struct Disk<'a> {
    pub name: &'a str,
}

pub type DiskAction<A> = Box<dyn Fn(Disk) -> (A, Disk)>;

pub fn unit<A: 'static>(a: A) -> DiskAction<A>
where
    A: Copy,
{
    Box::new(move |d| (a, d))
}

pub fn map<A: 'static, B: 'static>(d: DiskAction<A>, f: Box<dyn Fn(A) -> B>) -> DiskAction<B> {
    Box::new(move |disk| {
        let (a, b) = d(disk);
        (f(a), b)
    })
}

pub fn flatMap<A: 'static, B: 'static>(
    d: DiskAction<A>,
    f: Box<dyn Fn(A) -> DiskAction<B>>,
) -> DiskAction<B> {
    flatten(map(d, f))
}

pub fn flatten<A: 'static>(d: DiskAction<DiskAction<A>>) -> DiskAction<A> {
    Box::new(move |disk| {
        let (disk_action, disk2) = d(disk);
        disk_action(disk2)
    })
}

// pub fn unwrap<A>(d: DiskAction<Option<A>>) -> Option<DiskAction<A>> {
//     None
// }
