pub fn map_classes_to_tag(classes: &str) -> Option<&'static str> {
    if classes.contains("keyword") {
        Some("b")
    } else if classes.contains("string") {
        Some("i")
    } else if classes.contains("comment") {
        Some("s")
    } else if classes.contains("number") || classes.contains("constant.numeric") {
        Some("u")
    } else if classes.contains("operator") {
        Some("mark")
    } else if classes.contains("entity.name") || classes.contains("identifier") {
        Some("em")
    } else {
        None
    }
}
