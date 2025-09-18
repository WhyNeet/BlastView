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
    SetText {
        selector: String,
        text: String,
    },
    AppendChild {
        selector: String,
        html: String,
    },
    PrependChild {
        selector: String,
        html: String,
    },
    RemoveElement {
        selector: String,
    },
    Batch {
        patches: Vec<Patch>,
    },
}
