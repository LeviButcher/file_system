use super::disk::*;

static MAX_DATA_SIZE: u32 = 50;

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

pub fn remove_options<A>(b: Vec<Option<A>>) -> Vec<A> {
    b.into_iter()
        .fold(Vec::<A>::new(), |mut acc, curr| match curr {
            Some(a) => {
                acc.push(a);
                acc
            }
            None => acc,
        })
}

pub fn string_to_block_data_chunks(s: String) -> Vec<String> {
    let s = s.chars().collect::<Vec<char>>();
    s.chunks(MAX_DATA_SIZE as usize)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_to_block_data_chunks_return_expected() {
        let r = string_to_block_data_chunks(
            " This is my stirn go fahst ea;lsf jasjfadklsjfal;sdfjads f".into(),
        );
        assert_eq!(r.len(), 2);
    }
}
