use blastview::{component::Component, node::Node, renderer};

#[test]
fn component_rendering_works() {
    struct MyComponent;
    impl Component for MyComponent {
        fn render() -> impl Into<blastview::node::Node> {
            Node::new("div")
                .attr("class", "container")
                .child("Hello world!")
        }
    }

    let output = renderer::render_component_to_string::<MyComponent>();
    assert_eq!(output, r#"<div class="container">Hello world!</div>"#);
}
