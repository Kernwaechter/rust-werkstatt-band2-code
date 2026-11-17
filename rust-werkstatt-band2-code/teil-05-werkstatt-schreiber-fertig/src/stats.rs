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
    fn test_woerter() {
        assert_eq!(woerter_zaehlen("Hallo Werkstatt Schreiber"), 3);
        assert_eq!(woerter_zaehlen(""), 0);
    }

    #[test]
    fn test_zeichen() {
        assert_eq!(zeichen_zaehlen("Rust\n"), 4);
    }

    #[test]
    fn test_lesezeit() {
        assert_eq!(lesezeit_minuten(150), 1);
        assert_eq!(lesezeit_minuten(201), 2);
    }
}
