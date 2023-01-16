use super::low_link::LowLink;
use petgraph::{EdgeType, Graph};

/// Convert a graph to a biconnected graph by adding edges between vertexes.
pub fn make_biconnected<N, E, Ty>(graph: &mut Graph<N, E, Ty>)
where
    N: Copy + PartialEq,
    E: Default,
    Ty: EdgeType,
{
    loop {
        let mut low_link = LowLink::new(&*graph);
        low_link.traverse(&*graph);

        if low_link.articulations.len() == 0 {
            return;
        }

        // brute-force: pick non-adjacent 2 vertexes from a graph and connect them if
        // both are not an articulation.

        for n in graph.node_indices() {
            for m in graph.node_indices() {
                if n == m || graph.contains_edge(n, m) {
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

                graph.add_edge(n, m, E::default());
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::prelude::UnGraph;

    #[test]
    fn test_make_biconnected() {
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

        // -- convert it to biconnected graph
        make_biconnected(&mut g);

        let mut low_link = LowLink::new(&g);

        low_link.traverse(&g);

        assert_eq!(low_link.articulations.len(), 0);
        assert_eq!(low_link.bridges.len(), 0);
    }

    #[test]
    fn test_make_biconnected_2() {
        // ```ignore
        // v0- - - -(v1)- - - -(v2)- - - -v3
        // ```
        let mut g: UnGraph<&str, &str> = UnGraph::<&str, &str>::default();

        let v0 = g.add_node("v0");
        let v1 = g.add_node("v1");
        let v2 = g.add_node("v2");
        let v3 = g.add_node("v3");

        g.extend_with_edges(&[(v0, v1), (v1, v2), (v2, v3)]);

        // -- convert it to biconnected graph
        make_biconnected(&mut g);

        let mut low_link = LowLink::new(&g);

        low_link.traverse(&g);

        assert_eq!(low_link.articulations.len(), 0);
        assert_eq!(low_link.bridges.len(), 0);
    }

    #[test]
    fn test_make_biconnected_3() {
        // ```ignore
        //     6
        //     |
        //  0--1--2--3
        //     |
        //     4
        //     |
        //     5
        // ```
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

        let mut low_link = LowLink::new(&g);

        low_link.traverse(&g);

        assert_eq!(low_link.articulations.len(), 0);
        assert_eq!(low_link.bridges.len(), 0);
    }
}
