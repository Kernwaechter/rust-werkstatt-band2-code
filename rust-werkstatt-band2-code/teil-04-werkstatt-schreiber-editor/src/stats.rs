pub(crate) fn woerter_zaehlen(text: &str) -> usize {
    text.split_whitespace().count()
}

pub(crate) fn zeichen_zaehlen(text: &str) -> usize {
    text.chars()
        .filter(|c| *c != '\r' && *c != '\n')
        .count()
}

pub(crate) fn lesezeit_minuten(woerter: usize) -> usize {
    woerter.div_ceil(200)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_woerter_zaehlen() {
        assert_eq!(woerter_zaehlen("Hallo Werkstatt"), 2);
    }
}
