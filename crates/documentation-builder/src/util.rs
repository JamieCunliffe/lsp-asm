use itertools::Itertools;

pub fn range_to_list(range: &str) -> Option<Vec<String>> {
    let split = if let Some(stripped) = range.strip_prefix('-') {
        stripped.find('-').map(|x| x + 1)
    } else {
        range.find('-')
    };

    if let Some(split) = split {
        let start = &range[..split];
        let end = &range[(split + 1)..];

        let start = start.parse::<i32>().ok()?;
        let end = end.parse::<i32>().ok()?;

        Some((start..=end).map(|x| x.to_string()).collect_vec())
    } else {
        Some(vec![range.to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_range_parsing() {
        assert_eq!(
            range_to_list("0-63"),
            Some((0..=63).map(|x| x.to_string()).collect_vec())
        );

        assert_eq!(
            range_to_list("-64-64"),
            Some((-64..=64).map(|x| x.to_string()).collect_vec())
        );

        assert_eq!(range_to_list("-4"), Some(vec![String::from("-4")]));
    }

    #[test]
    fn test_range_no_number() {
        assert_eq!(range_to_list("0-number of something"), None);
    }
}
