use std::cmp::Ordering;
use std::collections::HashMap;

fn get_lms_starts<Char: Copy + Ord>(s: &[Char]) -> Vec<usize> {
    let mut result = Vec::new();
    if let Some((&c, s)) = s.split_last() {
        // The last character is always greater than the empty suffix.
        let mut next_is_s = false;
        let mut next_c = c;
        for (idx, &c) in s.iter().enumerate().rev() {
            let is_s = match c.cmp(&next_c) {
                Ordering::Less => true,
                Ordering::Equal => next_is_s,
                Ordering::Greater => false,
            };
            next_c = c;
            if !is_s && next_is_s {
                result.push(idx + 1);
            }
            next_is_s = is_s;
        }
        result.reverse();
    }
    result
}

/// Find the start or end of each bucket.
fn get_buckets<Char: Copy + Into<usize>>(s: &[Char], bkt: &mut [usize], end: bool) {
    // Compute the size of each bucket.
    bkt.fill(0);
    for &c in s {
        bkt[c.into()] += 1;
    }

    let mut sum = 0;
    for bin in bkt.iter_mut() {
        let cur = *bin;

        if end {
            sum += cur;
            *bin = sum;
        } else {
            *bin = sum;
            sum += cur;
        }
    }
}

fn induce_sa_l<Char: Copy + Ord + Into<usize>>(
    sa: &mut [usize],
    s: &[Char],
    bkt: &mut [usize],
    only_lms: bool,
) {
    get_buckets(s, bkt, false);

    // unroll once for implicit end-of-string marker
    if !s.is_empty() {
        let j = s.len() - 1;
        let bin = &mut bkt[s[j].into()];
        sa[*bin] = j;
        *bin += 1;
    }

    for i in 0..sa.len() {
        if sa[i].wrapping_add(1) >= 2 {
            let j = sa[i] - 1;
            if s[j] >= s[j + 1] {
                if only_lms {
                    sa[i] = usize::MAX;
                }
                let bin = &mut bkt[s[j].into()];
                sa[*bin] = j;
                *bin += 1;
            }
        }
    }
}

fn induce_sa_s<Char: Copy + Ord + Into<usize>>(
    sa: &mut [usize],
    s: &[Char],
    bkt: &mut [usize],
    only_lms: bool,
) {
    get_buckets(s, bkt, true);
    for i in (0..sa.len()).rev() {
        if sa[i].wrapping_add(1) >= 2 {
            let j = sa[i] - 1;
            if s[j] <= s[j + 1] {
                if only_lms {
                    sa[i] = usize::MAX;
                }
                let bin = &mut bkt[s[j].into()];
                *bin -= 1;
                sa[*bin] = j;
            }
        }
    }
}

fn sais_inner<Char: Copy + Ord + Into<usize> + TryFrom<usize>>(
    s: &[Char],
    sa: &mut [usize],
    k: usize,
) {
    sa.fill(usize::MAX);
    let lms_starts = get_lms_starts(s);

    {
        let mut bkt = vec![0; k + 1];
        get_buckets(s, &mut bkt, true);

        for &idx in lms_starts.iter() {
            let bin = &mut bkt[s[idx].into()];
            *bin -= 1;
            sa[*bin] = idx;
        }

        induce_sa_l(sa, s, &mut bkt, true);
        induce_sa_s(sa, s, &mut bkt, true);
    }

    let mut n1 = 0;
    for i in 0..sa.len() {
        if sa[i].wrapping_add(1) >= 2 {
            sa[n1] = sa[i];
            n1 += 1;
        }
    }
    sa[n1..].fill(usize::MAX);

    let mut next_start = s.len();
    let length = |&start| {
        let length = next_start - start;
        next_start = start;
        (start, length)
    };
    let lms_lengths: HashMap<_, _> = lms_starts.iter().rev().map(length).collect();

    let mut name = 1;
    let mut prev = usize::MAX;
    for i in 0..n1 {
        let pos = sa[i];
        if prev != usize::MAX {
            let end = pos + lms_lengths[&pos];
            let prev_end = prev + lms_lengths[&prev];
            if end.max(prev_end) == s.len() || s[pos..=end] != s[prev..=prev_end] {
                name += 1;
            }
        }
        prev = pos;
        let pos = pos / 2;
        sa[n1 + pos] = name - 1;
    }

    drop(lms_lengths);

    let mut j = sa.len();
    for i in (n1..sa.len()).rev() {
        if sa[i] != usize::MAX {
            j -= 1;
            sa[j] = sa[i];
        }
    }

    if name < n1 {
        let (sa1, s1) = sa.split_at_mut(sa.len() - n1);
        sais_inner(s1, &mut sa1[..n1], name - 1);
    } else {
        for i in 0..n1 {
            sa[sa[sa.len() - n1 + i]] = i;
        }
    }

    for i in 0..n1 {
        sa[i] = lms_starts[sa[i]];
    }
    sa[n1..].fill(usize::MAX);

    {
        let mut bkt = vec![0; k + 1];
        get_buckets(s, &mut bkt, true);

        for i in (0..n1).rev() {
            let j = sa[i];
            sa[i] = usize::MAX;
            let bin = &mut bkt[s[j].into()];
            *bin -= 1;
            sa[*bin] = j;
        }

        induce_sa_l(sa, s, &mut bkt, false);
        induce_sa_s(sa, s, &mut bkt, false);
    }
}

pub fn sais<Char: Copy + Ord + Into<usize> + TryFrom<usize>>(s: &[Char]) -> Vec<usize> {
    let mut sa = vec![usize::MAX; s.len()];
    let zero = 0usize.try_into().unwrap_or_else(|_| {
        panic!(
            "expected a zero-equivalent for {}",
            std::any::type_name::<Char>()
        )
    });
    sais_inner(s, &mut sa, s.iter().copied().max().unwrap_or(zero).into());
    sa
}

pub fn sais_utf8(s: &str) -> Vec<usize> {
    let bytes = s.as_bytes();
    let mut sa = sais(bytes);
    let start = sa.partition_point(|&i| bytes[i] < 0x80);
    let end = sa.partition_point(|&i| bytes[i] < 0xC0);
    sa.drain(start..end);
    sa
}
