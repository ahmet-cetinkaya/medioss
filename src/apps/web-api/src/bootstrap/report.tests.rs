use super::with_causes;

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
struct Link {
    message: &'static str,
    #[source]
    source: Option<Box<Link>>,
}

fn link(message: &'static str, source: Option<Link>) -> Link {
    Link {
        message,
        source: source.map(Box::new),
    }
}

#[test]
fn with_causes_renders_a_single_error_as_its_own_message() {
    assert_eq!(with_causes(&link("a", None)), "a");
}

#[test]
fn with_causes_appends_each_source_in_order() {
    let chained = link("a", Some(link("b", Some(link("c", None)))));

    assert_eq!(with_causes(&chained), "a: b: c");
}
