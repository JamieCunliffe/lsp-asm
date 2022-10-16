use dissimilar::Chunk;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub text: String,
    pub start: usize,
    pub end: Option<usize>,
}

pub fn diff(original: &str, new: &str) -> Vec<Change> {
    let mut position = 0;
    let mut ret = Vec::new();

    let diffs = dissimilar::diff(original, new);
    let mut diffs = diffs.iter().peekable();
    while let Some(diff) = diffs.next() {
        match (diff, diffs.peek()) {
            (Chunk::Delete(del), Some(Chunk::Insert(ins))) => {
                ret.push(Change {
                    text: ins.to_string(),
                    start: position,
                    end: Some(position + del.len()),
                });

                diffs.next().unwrap();
                position += del.len();
            }
            (Chunk::Equal(c), _) => position += c.len(),
            (Chunk::Delete(c), _) => {
                ret.push(Change {
                    text: String::from(""),
                    start: position as _,
                    end: Some(position + c.len()),
                });
                position += c.len();
            }
            (Chunk::Insert(c), _) => {
                ret.push(Change {
                    text: c.to_string(),
                    start: position,
                    end: None,
                });
            }
        }
    }

    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_simple_text_addition() {
        let original = "some text";
        let new = "some more text";
        let diff = diff(original, new);

        let expected = vec![Change {
            text: "more ".into(),
            start: 5,
            end: None,
        }];
        assert_eq!(diff, expected);
    }

    #[test]
    fn test_simple_text_delete() {
        let original = "some more text";
        let new = "some text";
        let diff = diff(original, new);

        let expected = vec![Change {
            text: "".into(),
            start: 5,
            end: Some(10),
        }];
        assert_eq!(diff, expected);
    }

    #[test]
    fn test_no_change() {
        let original = "some text";
        let new = "some text";
        let diff = diff(original, new);

        let expected = vec![];
        assert_eq!(diff, expected);
    }

    #[test]
    fn test_multiple_changes() {
        let original = "some more text";
        let new = "some text\nbut added this";
        let diff = diff(original, new);

        let expected = vec![Change {
            text: "text\nbut added this".into(),
            start: 5,
            end: Some(14),
        }];

        assert_eq!(diff, expected);
    }
}
