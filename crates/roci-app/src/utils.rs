pub trait Truncate {
    fn truncated(&self, max: usize) -> String;
}

impl Truncate for String {
    fn truncated(&self, max: usize) -> String {
        const ELLIPSIS: &str = "...";

        if self.chars().count() <= max {
            return self.to_string();
        }

        let keep = max - ELLIPSIS.len();
        let truncated: String = self.chars().take(keep).collect();
        format!("{truncated}{ELLIPSIS}")
    }
}
