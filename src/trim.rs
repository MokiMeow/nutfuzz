pub fn trim_coverage<F>(input: &[u8], mut preserves: F) -> Vec<u8>
where
    F: FnMut(&[u8]) -> bool,
{
    if !preserves(input) {
        return input.to_vec();
    }

    let mut best = input.to_vec();
    let mut chunk = best.len() / 2;
    while chunk > 0 {
        let mut offset = 0;
        while offset < best.len() && best.len() > 1 {
            let end = (offset + chunk).min(best.len());
            let mut candidate = best.clone();
            candidate.drain(offset..end);
            if !candidate.is_empty() && preserves(&candidate) {
                best = candidate;
            } else {
                offset += chunk;
            }
        }
        chunk /= 2;
    }
    best
}

#[cfg(test)]
mod tests {
    use super::trim_coverage;

    #[test]
    fn removes_bytes_that_do_not_affect_the_observed_path() {
        let trimmed = trim_coverage(b"xxxxKEYyyyy", |candidate| {
            candidate.windows(3).any(|window| window == b"KEY")
        });
        assert_eq!(trimmed, b"KEY");
    }
}
