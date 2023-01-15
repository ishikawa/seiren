use petgraph::visit::{
    GraphRef, IntoNeighbors, IntoNodeIdentifiers, NodeIndexable, VisitMap, Visitable,
};

// structure to enumerate articulations and bridges of a graph
//
// - DFS tree: starting from a vertex v ∈ V, DFS is performed so that each vertex is visited at most
// once. The tree consisting of the edges used is called a DFS tree. This tree is a rooted tree with
// root v
// - Backward edges: edges not used in the DFS tree in the direction from the leaf to the root
#[derive(Debug)]
pub struct LowLink<N, VM> {
    used: VM,
    // Order in which the vertices were visited in the DFS
    ord: Vec<usize>,
    // Minimum ord of vertices reachable from vertex v through the leafwise edge of the DFS tree more than 0 times and less than once through the backward edge
    low: Vec<usize>,
    pub articulations: Vec<N>,
    pub bridges: Vec<(N, N)>,
}

impl<N, VM> LowLink<N, VM>
where
    N: Copy + PartialEq,
    VM: VisitMap<N>,
{
    pub fn new<G>(graph: G) -> Self
    where
        G: GraphRef + NodeIndexable + Visitable<NodeId = N, Map = VM>,
    {
        let capacity = graph.node_bound();

        Self {
            used: graph.visit_map(),
            ord: vec![usize::MAX; capacity],
            low: vec![usize::MAX; capacity],
            articulations: vec![],
            bridges: vec![],
        }
    }

    pub fn traverse<G>(&mut self, graph: G)
    where
        G: IntoNeighbors<NodeId = N> + IntoNodeIdentifiers<NodeId = N> + NodeIndexable,
    {
        let mut k = 0;
        for node_id in graph.node_identifiers() {
            if !self.used.is_visited(&node_id) {
                k = self.dfs(graph, node_id, k, None);
            }
        }
    }

    fn dfs<G>(&mut self, graph: G, node: N, mut k: usize, parent: Option<N>) -> usize
    where
        G: IntoNeighbors<NodeId = N> + NodeIndexable,
    {
        let mut is_articulation = false;
        let idx = graph.to_index(node);
        let mut cnt = 0;

        self.used.visit(node);
        self.ord[idx] = k;
        self.low[idx] = self.ord[idx];
        k += 1;

        for to_node in graph.neighbors(node) {
            let to_idx = graph.to_index(to_node);

            if !self.used.is_visited(&to_node) {
                cnt += 1;
                k = self.dfs(graph, to_node, k, Some(node));
                self.low[idx] = self.low[idx].min(self.low[to_idx]);

                if parent.is_some() && self.ord[idx] <= self.low[to_idx] {
                    is_articulation = true;
                }

                if self.ord[idx] < self.low[to_idx] {
                    // bridge
                    if idx < to_idx {
                        self.bridges.push((node, to_node));
                    } else {
                        self.bridges.push((to_node, node));
                    }
                }
            } else if parent.filter(|p| *p == to_node).is_none() {
                // backward edges
                self.low[idx] = self.low[idx].min(self.ord[to_idx]);
            }
        }

        if parent.is_none() && cnt >= 2 {
            is_articulation = true;
        }
        if is_articulation {
            self.articulations.push(node);
        }

        k
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::prelude::UnGraph;

    #[test]
    fn low_link() {
        // Build a graph
        //
        // v0----.
        // |     |
        // v1    |
        // |     |
        // v4    v2
        //  \   /
        //   \ /
        //    v5 <- articulation
        //    | <-- bridge
        //    v3
        let mut g: UnGraph<&str, &str> = UnGraph::<&str, &str>::default();

        let v0 = g.add_node("v0");
        let v1 = g.add_node("v1");
        let v2 = g.add_node("v2");
        let v3 = g.add_node("v3");
        let v4 = g.add_node("v4");
        let v5 = g.add_node("v5");

        g.extend_with_edges(&[(v0, v1), (v0, v2), (v1, v4), (v4, v5), (v2, v5), (v5, v3)]);

        // -- Low Link

        let mut low_link = LowLink::new(&g);

        low_link.traverse(&g);

        println!("Low Link = {:?}", low_link);

        assert_eq!(low_link.articulations.len(), 1);
        assert_eq!(low_link.articulations[0], v5);
        assert_eq!(low_link.bridges.len(), 1);
        assert_eq!(low_link.bridges[0], (v3, v5));
    }
}