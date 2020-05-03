#[derive(Copy, Debug, Clone)]
pub struct Disk<'a> {
    pub file: &'a str,
    pub reads: u32,
    pub writes: u32,
}

impl<'a> Disk<'a> {
    pub fn new(file_name: &str) -> Disk {
        Disk {
            file: file_name,
            reads: 0,
            writes: 0,
        }
    }
    pub fn read(self) -> Disk<'a> {
        Disk {
            file: self.file,
            reads: self.reads + 1,
            writes: self.writes,
        }
    }
    pub fn write(self) -> Disk<'a> {
        Disk {
            file: self.file,
            reads: self.reads,
            writes: self.writes + 1,
        }
    }
}

pub type DiskAction<'a, A> = Box<dyn Fn(Disk) -> (A, Disk) + 'a>;

pub fn unit<'a, A: 'a>(a: A) -> DiskAction<'a, A>
where
    A: Clone,
{
    Box::new(move |d| (a.clone(), d))
}

pub fn map<'a, A: 'a, B: 'a>(d: DiskAction<'a, A>, f: Box<dyn Fn(A) -> B>) -> DiskAction<'a, B> {
    Box::new(move |disk| {
        let (a, b) = d(disk);
        (f(a), b)
    })
}

pub fn flat_map<'a, A: 'a, B: 'a>(
    d: DiskAction<'a, A>,
    f: Box<dyn Fn(A) -> DiskAction<'a, B>>,
) -> DiskAction<'a, B> {
    flatten(map(d, f))
}

pub fn flatten<'a, A: 'a>(d: DiskAction<'a, DiskAction<'a, A>>) -> DiskAction<'a, A> {
    Box::new(move |disk| {
        let (disk_action, disk2) = d(disk);
        disk_action(disk2)
    })
}

pub fn map2<'a, A: 'a, B: 'a, C: 'a>(
    d: DiskAction<'a, A>,
    d2: DiskAction<'a, B>,
    f: Box<dyn Fn(A, B) -> C + 'a>,
) -> DiskAction<'a, C> {
    Box::new(move |disk| {
        let (res, disk2) = d(disk);
        let (res2, disk3) = d2(disk2);
        (f(res, res2), disk3)
    })
}

pub fn flatten_option<'a, A: 'a>(
    d: DiskAction<'a, Option<Option<A>>>,
) -> DiskAction<'a, Option<A>> {
    map(d, Box::new(|x| x.flatten()))
}

pub fn sequence<'a, A: 'a>(a: Vec<DiskAction<'a, A>>) -> DiskAction<'a, Vec<A>>
where
    A: Clone,
{
    a.into_iter().fold(unit(Vec::<A>::new()), |acc, curr| {
        map2(
            acc,
            curr,
            Box::new(|mut vec: Vec<A>, x: A| {
                vec.push(x);
                vec
            }),
        )
    })
}
