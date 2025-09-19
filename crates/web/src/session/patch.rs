use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Patch {
    ReplaceInner {
        selector: String,
        html: String,
    },
    ReplaceOuter {
        selector: String,
        html: String,
    },
    SetAttribute {
        selector: String,
        name: String,
        value: String,
    },
    RemoveAttribute {
        selector: String,
        name: String,
    },
    AppendChild {
        selector: String,
        html: String,
    },
    PrependChild {
        selector: String,
        html: String,
    },
    ReplaceChild {
        selector: String,
        index: usize,
        html: String,
    },
    RemoveElement {
        selector: String,
    },
    AttachEvent {
        selector: String,
        event: String,
    },
    DetachEvent {
        selector: String,
        event: String,
    },
    Batch {
        patches: Vec<Patch>,
    },
}
