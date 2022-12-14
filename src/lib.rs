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

/// Find the end of each bucket.
fn get_buckets<Char: Copy + Into<usize>>(s: &[Char], bkt: &mut [usize]) {
    // Compute the size of each bucket.
    bkt.fill(0);
    for &c in s {
        bkt[c.into()] += 1;
    }

    let mut sum = 0;
    for bin in bkt.iter_mut() {
        sum += *bin;
        *bin = sum;
    }
}

fn induce_sort<Char: Copy + Ord + Into<usize>>(
    sa: &mut [usize],
    s: &[Char],
    bkt: &mut [usize],
    only_lms: bool,
) {
    let mut start = std::mem::replace(&mut bkt[0], 0);
    for bin in bkt[1..].iter_mut() {
        // Either the next bin is full, in which case it already points to its start; or there's at
        // least one unfilled position at the start of that bin and we just have to find it.
        while start < *bin && sa[start] != 0 {
            start += 1;
        }
        std::mem::swap(&mut start, bin);
    }
    debug_assert_eq!(start, sa.len());

    // unroll once for implicit end-of-string marker
    if !s.is_empty() {
        let j = s.len() - 1;
        let bin = &mut bkt[s[j].into()];
        sa[*bin] = j;
        *bin += 1;
    }

    for i in 0..sa.len() {
        if sa[i] != 0 {
            let j = sa[i] - 1;
            if s[j] >= s[j + 1] {
                if only_lms {
                    sa[i] = 0;
                }
                let bin = &mut bkt[s[j].into()];
                sa[*bin] = j;
                *bin += 1;
            }
        }
    }

    get_buckets(s, bkt);
    for i in (0..sa.len()).rev() {
        if sa[i] != 0 {
            let j = sa[i] - 1;
            if s[j] <= s[j + 1] {
                if only_lms {
                    sa[i] = 0;
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
    let lms_starts = get_lms_starts(s);

    {
        let mut bkt = vec![0; k + 1];
        get_buckets(s, &mut bkt);

        for &idx in lms_starts.iter() {
            let bin = &mut bkt[s[idx].into()];
            *bin -= 1;
            sa[*bin] = idx;
        }

        induce_sort(sa, s, &mut bkt, true);
    }

    let mut n1 = 0;
    for i in 0..sa.len() {
        if sa[i] != 0 {
            let lms = sa[i];
            sa[i] = 0;
            sa[n1] = lms;
            n1 += 1;
        }
    }

    let (sa1, s1) = sa.split_at_mut(n1);

    let mut name = 0;
    if let Some((&pos, sa1)) = sa1.split_first() {
        let mut next_start = s.len();
        let length = |&start| {
            let length = next_start - start;
            next_start = start;
            (start, length)
        };
        let lms_lengths: HashMap<_, _> = lms_starts.iter().rev().map(length).collect();

        name = 1;
        let mut prev = pos;
        s1[pos / 2] = name;
        for &pos in sa1.iter() {
            let end = pos + lms_lengths[&pos];
            let prev_end = prev + lms_lengths[&prev];
            if end.max(prev_end) == s.len() || s[pos..=end] != s[prev..=prev_end] {
                name += 1;
            }
            prev = pos;
            s1[pos / 2] = name;
        }
    }

    if name < n1 {
        let mut j = 0;
        for i in 0..s1.len() {
            if s1[i] != 0 {
                let name = s1[i] - 1;
                s1[i] = 0;
                s1[j] = name;
                j += 1;
            }
        }

        debug_assert_eq!(j, n1);
        let s1 = &mut s1[..n1];

        sa1.fill(0);
        sais_inner(s1, sa1, name - 1);
        s1.fill(0);

        for lms in sa1 {
            *lms = lms_starts[*lms];
        }
    } else {
        debug_assert_eq!(name, n1);
        s1.fill(0);
    }

    {
        let mut bkt = vec![0; k + 1];
        get_buckets(s, &mut bkt);

        for i in (0..n1).rev() {
            let j = sa[i];
            sa[i] = 0;
            let bin = &mut bkt[s[j].into()];
            *bin -= 1;
            sa[*bin] = j;
        }

        induce_sort(sa, s, &mut bkt, false);
    }
}

pub fn sais<Char: Copy + Ord + Into<usize> + TryFrom<usize>>(s: &[Char]) -> Vec<usize> {
    let mut sa = vec![0; s.len()];
    sais_inner(s, &mut sa, s.iter().copied().max().map_or(0, Into::into));
    sa
}

pub fn sais_utf8(s: &str) -> Vec<usize> {
    let bytes = s.as_bytes();
    let mut sa = sais(bytes);
    let start = sa.partition_point(|&i| bytes[i] < 0x80);
    let length = sa[start..].partition_point(|&i| bytes[i] < 0xC0);
    sa.drain(start..start + length);
    sa
}
