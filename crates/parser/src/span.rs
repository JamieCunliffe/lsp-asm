use std::str::{CharIndices, Chars};

use nom::{Compare, FindSubstring, InputIter, InputLength, InputTake, Needed, UnspecializedInput};

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Span<'a, T> {
    data: &'a str,
    extra: T,
}

impl<'a, T> Span<'a, T> {
    pub fn new(data: &'a str, extra: T) -> Self {
        Self { data, extra }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn chars(&self) -> Chars<'_> {
        self.data.chars()
    }

    pub fn extra(&self) -> &T {
        &self.extra
    }

    pub fn as_str(&self) -> &'a str {
        self.data
    }
}

impl<'a, T> InputLength for Span<'a, T> {
    fn input_len(&self) -> usize {
        self.data.input_len()
    }
}

impl<'a, T> InputIter for Span<'a, T> {
    type Item = char;
    type Iter = CharIndices<'a>;
    type IterElem = Chars<'a>;

    fn iter_indices(&self) -> Self::Iter {
        self.data.iter_indices()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.data.iter_elements()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.data.position(predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        self.data.slice_index(count)
    }
}

impl<'a, T: Clone> InputTake for Span<'a, T> {
    fn take(&self, count: usize) -> Self {
        Self {
            data: self.data.take(count),
            extra: self.extra.clone(),
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let res = self.data.take_split(count);
        (
            Self {
                data: res.0,
                extra: self.extra.clone(),
            },
            Self {
                data: res.1,
                extra: self.extra.clone(),
            },
        )
    }
}

impl<'a, T> UnspecializedInput for Span<'a, T> {}

impl<'a, T> Compare<&'a str> for Span<'a, T> {
    fn compare(&self, t: &'a str) -> nom::CompareResult {
        self.data.compare(t)
    }

    fn compare_no_case(&self, t: &'a str) -> nom::CompareResult {
        self.data.compare_no_case(t)
    }
}

impl<'a, T> FindSubstring<&'static str> for Span<'a, T> {
    fn find_substring(&self, substr: &'static str) -> Option<usize> {
        self.as_str().find(substr)
    }
}
