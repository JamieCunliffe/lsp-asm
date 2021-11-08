use documentation::CompletionValue;

pub fn process_range(range: &str) -> Option<CompletionValue> {
    let split = if let Some(stripped) = range.strip_prefix('-') {
        stripped.find('-').map(|x| x + 1)
    } else {
        range.find('-')
    };

    if let Some(split) = split {
        let start = &range[..split];
        let end = &range[(split + 1)..];

        let start = start.parse::<i64>().ok()?;
        let end = end.parse::<i64>().ok()?;

        Some(CompletionValue::Right(start..(end + 1)))
    } else {
        Some(CompletionValue::Left(range.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_range_parsing() {
        assert_eq!(process_range("0-63"), Some(CompletionValue::Right(0..64)));

        assert_eq!(
            process_range("-64-64"),
            Some(CompletionValue::Right(-64..65))
        );

        assert_eq!(
            process_range("-4"),
            Some(CompletionValue::Left(String::from("-4")))
        );
    }

    #[test]
    fn test_range_no_number() {
        assert_eq!(process_range("0-number of something"), None);
    }
}
