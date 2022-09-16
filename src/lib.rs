use bitvec::vec::BitVec;
use std::cmp::Ordering;

fn classify<Char: Copy + Ord>(t: &mut BitVec, s: &[Char]) {
    t.clear();
    t.resize(s.len(), false);

    let mut next_is_s = true;
    let mut next_c = None;
    for (mut is_s, &c) in t.iter_mut().zip(s).rev() {
        next_is_s = match Some(c).cmp(&next_c) {
            Ordering::Less => true,
            Ordering::Equal => next_is_s,
            Ordering::Greater => false,
        };
        next_c = Some(c);
        is_s.set(next_is_s);
    }
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

fn is_lms(t: &BitVec, i: usize) -> bool {
    i > 0 && t[i] && !t[i - 1]
}

fn init_sa<Char: Copy + Into<usize>>(t: &BitVec, sa: &mut [usize], s: &[Char], bkt: &mut [usize]) {
    sa.fill(usize::MAX);
    for (idx, &c) in s.iter().enumerate() {
        if is_lms(&t, idx) {
            let bin = &mut bkt[c.into()];
            *bin -= 1;
            sa[*bin] = idx;
        }
    }
}

fn induce_sa_l<Char: Copy + Into<usize>>(
    t: &BitVec,
    sa: &mut [usize],
    s: &[Char],
    bkt: &mut [usize],
) {
    get_buckets(s, bkt, false);

    // unroll once for implicit end-of-string marker
    if !s.is_empty() {
        let j = s.len() - 1;
        debug_assert!(!t[j]);
        let bin = &mut bkt[s[j].into()];
        sa[*bin] = j;
        *bin += 1;
    }

    for i in 0..sa.len() {
        if sa[i].wrapping_add(1) >= 2 {
            let j = sa[i] - 1;
            if !t[j] {
                let bin = &mut bkt[s[j].into()];
                sa[*bin] = j;
                *bin += 1;
            }
        }
    }
}

fn induce_sa_s<Char: Copy + Into<usize>>(
    t: &BitVec,
    sa: &mut [usize],
    s: &[Char],
    bkt: &mut [usize],
) {
    get_buckets(s, bkt, true);
    for i in (0..sa.len()).rev() {
        if sa[i].wrapping_add(1) >= 2 {
            let j = sa[i] - 1;
            if t[j] {
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
    let mut t = BitVec::new();
    classify(&mut t, s);

    {
        let mut bkt = vec![0; k + 1];
        get_buckets(s, &mut bkt, true);
        init_sa(&t, sa, s, &mut bkt);
        induce_sa_l(&t, sa, s, &mut bkt);
        induce_sa_s(&t, sa, s, &mut bkt);
    }

    let mut n1 = 0;
    for i in 0..sa.len() {
        if is_lms(&t, sa[i]) {
            sa[n1] = sa[i];
            n1 += 1;
        }
    }
    sa[n1..].fill(usize::MAX);

    let mut name = 0;
    let mut prev = usize::MAX;
    for i in 0..n1 {
        let pos = sa[i];
        for d in 0.. {
            if prev == usize::MAX
                || pos.max(prev) + d == s.len()
                || s[pos + d] != s[prev + d]
                || t[pos + d] != t[prev + d]
            {
                name += 1;
                prev = pos;
                break;
            }
            if d > 0 && (is_lms(&t, pos + d) || is_lms(&t, prev + d)) {
                break;
            }
        }
        let pos = pos / 2;
        sa[n1 + pos] = name - 1;
    }

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

    {
        let mut bkt = vec![0; k + 1];
        get_buckets(s, &mut bkt, true);

        let mut j = sa.len() - n1;
        for i in 1..sa.len() {
            if is_lms(&t, i) {
                sa[j] = i.try_into().unwrap();
                j += 1;
            }
        }

        for i in 0..n1 {
            sa[i] = sa[sa.len() - n1 + sa[i]];
        }

        sa[n1..].fill(usize::MAX);

        for i in (0..n1).rev() {
            let j = sa[i];
            sa[i] = usize::MAX;
            let bin = &mut bkt[s[j].into()];
            *bin -= 1;
            sa[*bin] = j;
        }

        induce_sa_l(&t, sa, s, &mut bkt);
        induce_sa_s(&t, sa, s, &mut bkt);
    }
}

pub fn sais(s: &[u8]) -> Vec<usize> {
    let mut sa = vec![usize::MAX; s.len()];
    sais_inner(s, &mut sa, s.iter().copied().max().unwrap_or(0).into());
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
