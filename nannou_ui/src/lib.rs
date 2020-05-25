use graph::Graph;

pub use widget::Widget;

pub mod graph;
pub mod widget;

#[derive(Debug)]
pub struct Ui {
    /// Stores and describes the relationships of all widgets.
    widget_graph: Graph,
    /// The root node of the entire widget graph.
    ///
    /// The root node should always exist within the graph for the lifetime of the `Ui`. All other
    /// widgets are children of the root.
    ///
    /// The immediate children of the root node represent rectangular surfaces that may receive
    /// events or be rendered. This is most commonly useful for windows.
    root: widget::Id,
}

pub enum InputEvent {}

pub struct RenderCommands {}

impl Ui {
    pub fn new() -> Self {
        let mut widget_graph = Graph::new();

        // Add the root of the entire UI.
        let root = widget_graph.add_node(widget::Root.into());

        Self {
            widget_graph,
            root,
        }
    }

    /// The **root** widget of the **Ui**.
    ///
    /// The **root** is the ancestor of all other widgets within the **Ui**'s widget graph.
    pub fn root(&self) -> widget::Id {
        self.root
    }

    /// Add the given widget as a child of the widget at the given parent ID.
    ///
    /// Returns a unique identifier associated with the given child.
    pub fn add_child<T>(&mut self, parent: widget::Id, child: T) -> widget::Id
    where
        T: Widget,
    {
        let node = graph::Node::from(child);
        let (_e_ix, w_id) = self.widget_graph.add_child(parent, node);
        w_id
    }

    pub fn process_input_event(&mut self, _branch: widget::Id, _event: InputEvent) {
        unimplemented!()
    }

    pub fn render(&self) -> RenderCommands {
        unimplemented!()
    }
}

impl Default for Ui {
    fn default() -> Self {
        Ui::new()
    }
}
