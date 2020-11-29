use super::Edge;
use super::GraphBuilder;
use std::iter::Sum;
pub trait Graphable {
    type Edge: Edge;

    fn is_node(&self, node: &<Self::Edge as Edge>::Node) -> bool;

    fn neighbors(
        &self,
        node: &<Self::Edge as Edge>::Node,
    ) -> Box<dyn Iterator<Item = (&<Self::Edge as Edge>::Node, &Self::Edge)>>;
}

pub fn graph<W, E, G>(g: &G) -> GraphBuilder<E, G>
where
    G: Graphable<Edge = E>,
    E: Edge<Weight = W>,
    W: Sum + Clone,
{
    GraphBuilder::new(g)
}
