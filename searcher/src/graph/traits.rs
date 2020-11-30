use super::Edge;

pub trait Graphable {
    type Edge: Edge;

    fn is_node(&self, node: &<Self::Edge as Edge>::Node) -> bool;

    fn neighbors(
        &self,
        node: &<Self::Edge as Edge>::Node,
    ) -> Vec<(<Self::Edge as Edge>::Node, Self::Edge)>;
}
