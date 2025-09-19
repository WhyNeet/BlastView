use uuid::Uuid;

use crate::node::Node;

#[derive(Debug, Clone)]
pub enum NodePatch {
    ReplaceChildren {
        node_id: Uuid,
        children: Vec<Node>,
    },
    ReplaceViewChildren {
        view_id: Uuid,
        children: Vec<Node>,
    },
    ReplaceChild {
        node_id: Uuid,
        child_idx: usize,
        node: Node,
    },
    Replace {
        node_id: Uuid,
        node: Node,
    },
    SetAttr {
        node_id: Uuid,
        attr: String,
        value: String,
    },
    RemoveAttr {
        node_id: Uuid,
        attr: String,
    },
    AttachEvent {
        node_id: Uuid,
        event: String,
    },
    DetachEvent {
        node_id: Uuid,
        event: String,
    },
}

pub fn diff(from_node: Node, to_node: Node, parent_id: Uuid, idx: usize) -> Vec<NodePatch> {
    match from_node {
        Node::Element(from) => match to_node {
            Node::Element(ref to) => {
                if from.tag != to.tag {
                    return vec![NodePatch::Replace {
                        node_id: from.id,
                        node: to_node,
                    }];
                }

                let mut patches = vec![];

                for (attr, value) in from.attrs.iter() {
                    if let Some(to_value) = to.attrs.get(attr) {
                        if value != to_value {
                            patches.push(NodePatch::SetAttr {
                                node_id: from.id,
                                attr: attr.to_string(),
                                value: to_value.to_string(),
                            });
                        }
                    } else {
                        patches.push(NodePatch::RemoveAttr {
                            node_id: from.id,
                            attr: attr.to_string(),
                        });
                    }
                }

                for (attr, value) in to.attrs.iter() {
                    if !from.attrs.contains_key(attr) {
                        patches.push(NodePatch::SetAttr {
                            node_id: from.id,
                            attr: attr.to_string(),
                            value: value.to_string(),
                        });
                    }
                }

                for (event, _) in from.events.iter() {
                    if !to.events.contains_key(event) {
                        patches.push(NodePatch::DetachEvent {
                            node_id: from.id,
                            event: event.to_string(),
                        });
                    }
                }

                for (event, _) in to.events.iter() {
                    if !from.events.contains_key(event) {
                        patches.push(NodePatch::AttachEvent {
                            node_id: from.id,
                            event: event.to_string(),
                        });
                    }
                }

                let parent_id = from.id;

                (*from)
                    .children
                    .into_iter()
                    .zip(
                        (match to_node {
                            Node::Element(node) => node.children,
                            _ => unreachable!(),
                        })
                        .into_iter(),
                    )
                    .for_each(|(from, to)| {
                        patches.append(&mut diff(from, to, parent_id, idx));
                    });

                patches
            }
            other => vec![NodePatch::Replace {
                node_id: from.id,
                node: other,
            }],
        },
        Node::Text(from) => match to_node {
            Node::Element(_) => vec![NodePatch::ReplaceChild {
                node_id: parent_id,
                child_idx: idx,
                node: to_node,
            }],
            Node::Text(ref to) => {
                if from.0 != to.0 {
                    vec![NodePatch::ReplaceChild {
                        node_id: parent_id,
                        child_idx: idx,
                        node: to_node,
                    }]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        },
        _ => vec![],
    }
}
