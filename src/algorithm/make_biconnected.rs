use std::collections::HashSet;

use super::low_link::LowLink;
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::{EdgeType, Graph};

/// Convert a graph to a biconnected graph by adding edges between vertexes.
///
/// A biconnected graph is a connected and "nonseparable" graph, meaning that if any one vertex were
/// to be removed, the graph will remain connected. Therefore a biconnected graph has no
/// articulation vertices. The property of being 2-connected is equivalent to biconnectivity, except
/// that the complete graph of two vertices is usually not regarded as 2-connected.
pub fn make_biconnected<N, E, Ty>(graph: &mut Graph<N, E, Ty>)
where
    N: Copy + PartialEq,
    E: Default,
    Ty: EdgeType,
{
    let mut ei: Option<EdgeIndex> = None;
    let mut n = 0;
    let mut s: HashSet<(NodeIndex, NodeIndex)> = HashSet::new();

    'LOOP: loop {
        let mut low_link = LowLink::new(&*graph);
        low_link.traverse(&*graph);

        if low_link.articulations.is_empty() {
            // The graph became biconnected
            return;
        } else if let Some(ei) = ei {
            if low_link.articulations.len() == n {
                graph.remove_edge(ei);
            }
        }

        n = low_link.articulations.len();

        // brute-force: pick non-adjacent 2 vertexes from a graph and connect them if
        // both are not an articulation.
        for n in graph.node_indices() {
            for m in graph.node_indices() {
                if n == m {
                    continue;
                }
                if graph.contains_edge(n, m) {
                    continue;
                }
                if low_link
                    .articulations
                    .iter()
                    .copied()
                    .any(|i| i == n || i == m)
                {
                    continue;
                }
                if s.contains(&(n, m)) {
                    continue;
                }

                s.insert((n, m));
                ei = Some(graph.add_edge(n, m, E::default()));

                // Re-check whether the graph became biconnected or not?
                continue 'LOOP;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::prelude::UnGraph;

    #[test]
    fn make_biconnected_1() {
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

        // -- convert it to biconnected graph
        make_biconnected(&mut g);
        assert_eq!(g.edge_count(), 7);
        g.contains_edge(v0, v3);

        let mut low_link = LowLink::new(&g);

        low_link.traverse(&g);

        assert_eq!(low_link.articulations.len(), 0);
        assert_eq!(low_link.bridges.len(), 0);
    }

    #[test]
    fn make_biconnected_2() {
        // v0- - - -(v1)- - - -(v2)- - - -v3
        let mut g: UnGraph<&str, &str> = UnGraph::<&str, &str>::default();

        let v0 = g.add_node("v0");
        let v1 = g.add_node("v1");
        let v2 = g.add_node("v2");
        let v3 = g.add_node("v3");

        g.extend_with_edges(&[(v0, v1), (v1, v2), (v2, v3)]);

        // -- convert it to biconnected graph
        make_biconnected(&mut g);
        assert_eq!(g.edge_count(), 4);
        g.contains_edge(v0, v3);

        let mut low_link = LowLink::new(&g);

        low_link.traverse(&g);

        assert_eq!(low_link.articulations.len(), 0);
        assert_eq!(low_link.bridges.len(), 0);
    }

    #[test]
    fn make_biconnected_3() {
        //     6
        //     |
        //  0--1--2--3
        //     |
        //     4
        //     |
        //     5
        let mut g: UnGraph<&str, &str> = UnGraph::<&str, &str>::default();

        let v0 = g.add_node("v0");
        let v1 = g.add_node("v1");
        let v2 = g.add_node("v2");
        let v3 = g.add_node("v3");
        let v4 = g.add_node("v4");
        let v5 = g.add_node("v5");
        let v6 = g.add_node("v6");

        g.extend_with_edges(&[(v0, v1), (v1, v2), (v2, v3), (v1, v4), (v4, v5), (v1, v6)]);

        // -- convert it to biconnected graph
        make_biconnected(&mut g);

        assert_eq!(g.edge_count(), 9);
        g.contains_edge(v0, v5);
        g.contains_edge(v0, v3);

        // low link
        let mut low_link = LowLink::new(&g);
        low_link.traverse(&g);

        assert_eq!(low_link.articulations.len(), 0);
        assert_eq!(low_link.bridges.len(), 0);
    }

    #[test]
    fn make_biconnected_empty() {
        let mut g: UnGraph<&str, &str> = UnGraph::<&str, &str>::default();

        make_biconnected(&mut g);

        let mut low_link = LowLink::new(&g);
        low_link.traverse(&g);

        assert_eq!(low_link.articulations.len(), 0);
        assert_eq!(low_link.bridges.len(), 0);
    }

    #[test]
    fn make_biconnected_one() {
        let mut g: UnGraph<&str, &str> = UnGraph::<&str, &str>::default();
        g.add_node("A");

        make_biconnected(&mut g);
        assert_eq!(g.edge_count(), 0);

        let mut low_link = LowLink::new(&g);
        low_link.traverse(&g);

        assert_eq!(low_link.articulations.len(), 0);
        assert_eq!(low_link.bridges.len(), 0);
    }

    #[test]
    fn make_biconnected_two() {
        // A graph of two vertices
        let mut g: UnGraph<&str, &str> = UnGraph::<&str, &str>::default();

        let v0 = g.add_node("v0");
        let v1 = g.add_node("v1");

        g.extend_with_edges(&[(v0, v1)]);
        assert_eq!(g.edge_count(), 1);

        make_biconnected(&mut g);

        let mut low_link = LowLink::new(&g);
        low_link.traverse(&g);

        assert_eq!(low_link.articulations.len(), 0);
        assert_eq!(low_link.bridges.len(), 1);
    }
}
