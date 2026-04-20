//! Mind Mapper — radial layout + SVG emission for tree-of-ideas visualisation.
//!
//! Second Graphics project. Introduces **radial auto-layout**: the root
//! at centre, children positioned in a circle around the root at
//! increasing radius, siblings spread evenly over their parent's
//! angular arc. Every mind-mapping tool (MindMup, XMind, SimpleMind,
//! iThoughts) uses some variant of this; the math is polar-to-cartesian
//! conversion over a recursive tree walk.
//!
//! Distinctive moves:
//!   * **Radial polar-to-cartesian layout** — depth → radius;
//!      sibling index → angle.
//!   * **Recursive tree-of-children model** (unlike G113's parent
//!      pointers — G115 stores children directly, which matches how
//!      mind-map UIs navigate).
//!   * **Collapsed-node state** — collapsing a node hides its subtree
//!      from layout and emission.
//!   * **SVG emission** — text output that a browser or SVG viewer
//!      renders directly. First "visual format" in the Rosetta Stone.

#[derive(Debug, Clone, PartialEq)]
pub struct MindNode {
    pub id: u64,
    pub label: String,
    pub color: String,         // hex string like "#3b82f6"
    pub collapsed: bool,
    pub children: Vec<MindNode>,
}

impl MindNode {
    pub fn new(id: u64, label: &str) -> Self {
        Self {
            id, label: label.into(),
            color: "#3b82f6".into(),
            collapsed: false,
            children: Vec::new(),
        }
    }

    pub fn with_color(mut self, color: &str) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_child(mut self, child: MindNode) -> Self {
        self.children.push(child);
        self
    }

    /// Find a node by ID in the subtree rooted at self.
    pub fn find(&self, id: u64) -> Option<&MindNode> {
        if self.id == id { return Some(self); }
        for c in &self.children {
            if let Some(n) = c.find(id) { return Some(n); }
        }
        None
    }

    pub fn find_mut(&mut self, id: u64) -> Option<&mut MindNode> {
        if self.id == id { return Some(self); }
        for c in &mut self.children {
            if let Some(n) = c.find_mut(id) { return Some(n); }
        }
        None
    }

    /// Toggle collapsed state on a node. Returns new state, or None if missing.
    pub fn toggle_collapse(&mut self, id: u64) -> Option<bool> {
        let n = self.find_mut(id)?;
        n.collapsed = !n.collapsed;
        Some(n.collapsed)
    }

    /// Total node count (visible only — collapsed subtrees omitted).
    pub fn visible_count(&self) -> usize {
        if self.collapsed { return 1; }
        1 + self.children.iter().map(|c| c.visible_count()).sum::<usize>()
    }

    /// Total node count ignoring collapse state.
    pub fn total_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.total_count()).sum::<usize>()
    }

    /// Max depth (0 for a leaf root).
    pub fn max_depth(&self) -> u32 {
        if self.children.is_empty() || self.collapsed { 0 }
        else { 1 + self.children.iter().map(|c| c.max_depth()).max().unwrap_or(0) }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaidOutNode {
    pub id: u64,
    pub label: String,
    pub color: String,
    pub depth: u32,
    pub x: f64,
    pub y: f64,
    pub parent_id: Option<u64>,
    pub collapsed: bool,
}

/// Radial layout: root at (cx, cy); children laid out in a circle around parent.
///
/// `radius_step` is the distance added per depth level. Siblings split their
/// parent's angular arc evenly. The root's arc is a full circle; at depth ≥ 1
/// each node inherits an arc from its parent.
pub fn radial_layout(root: &MindNode, cx: f64, cy: f64, radius_step: f64) -> Vec<LaidOutNode> {
    let mut out = Vec::new();
    out.push(LaidOutNode {
        id: root.id, label: root.label.clone(), color: root.color.clone(),
        depth: 0, x: cx, y: cy, parent_id: None, collapsed: root.collapsed,
    });
    if !root.collapsed {
        layout_children(root, cx, cy, 0.0, 2.0 * std::f64::consts::PI, 1, radius_step, cx, cy, &mut out);
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn layout_children(
    parent: &MindNode,
    parent_x: f64, parent_y: f64,
    arc_start: f64, arc_end: f64,
    depth: u32,
    radius_step: f64,
    cx: f64, cy: f64,
    out: &mut Vec<LaidOutNode>,
) {
    let n = parent.children.len();
    if n == 0 { return; }
    let slice = (arc_end - arc_start) / n as f64;
    for (i, child) in parent.children.iter().enumerate() {
        let angle_start = arc_start + slice * i as f64;
        let angle_end = arc_start + slice * (i + 1) as f64;
        let angle = (angle_start + angle_end) / 2.0;
        let r = depth as f64 * radius_step;
        let x = cx + r * angle.cos();
        let y = cy + r * angle.sin();
        out.push(LaidOutNode {
            id: child.id, label: child.label.clone(), color: child.color.clone(),
            depth, x, y, parent_id: Some(parent.id), collapsed: child.collapsed,
        });
        if !child.collapsed {
            layout_children(child, x, y, angle_start, angle_end, depth + 1,
                            radius_step, cx, cy, out);
        }
    }
    // Suppress unused warning; parent_x/parent_y kept for future edge-routing use.
    let _ = (parent_x, parent_y);
}

/// Pre-order DFS traversal of visible nodes.
pub fn traverse_dfs(root: &MindNode) -> Vec<u64> {
    let mut out = Vec::new();
    dfs_visit(root, &mut out);
    out
}

fn dfs_visit(node: &MindNode, out: &mut Vec<u64>) {
    out.push(node.id);
    if node.collapsed { return; }
    for c in &node.children {
        dfs_visit(c, out);
    }
}

/// Breadth-first traversal of visible nodes.
pub fn traverse_bfs(root: &MindNode) -> Vec<u64> {
    let mut out = Vec::new();
    let mut queue: Vec<&MindNode> = vec![root];
    while let Some(n) = queue.first().copied() {
        queue.remove(0);
        out.push(n.id);
        if !n.collapsed {
            for c in &n.children { queue.push(c); }
        }
    }
    out
}

/// Emit an SVG rendering of the laid-out map.
pub fn to_svg(nodes: &[LaidOutNode], width: f64, height: f64, radius: f64) -> String {
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">\n"
    );
    // Edges: parent → child lines.
    for node in nodes {
        if let Some(parent_id) = node.parent_id {
            if let Some(parent) = nodes.iter().find(|n| n.id == parent_id) {
                out.push_str(&format!(
                    "  <line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"#cbd5e1\" stroke-width=\"2\"/>\n",
                    parent.x, parent.y, node.x, node.y
                ));
            }
        }
    }
    // Nodes: circles with labels.
    for node in nodes {
        out.push_str(&format!(
            "  <circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{radius}\" fill=\"{}\"/>\n",
            node.x, node.y, node.color
        ));
        out.push_str(&format!(
            "  <text x=\"{:.2}\" y=\"{:.2}\" text-anchor=\"middle\" dy=\"0.35em\" fill=\"white\" font-size=\"12\">{}</text>\n",
            node.x, node.y, escape_xml(&node.label)
        ));
    }
    out.push_str("</svg>\n");
    out
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> MindNode {
        MindNode::new(1, "Rosetta Stone")
            .with_child(
                MindNode::new(2, "Languages")
                    .with_child(MindNode::new(5, "Rust"))
                    .with_child(MindNode::new(6, "Python"))
                    .with_child(MindNode::new(7, "Lean"))
            )
            .with_child(
                MindNode::new(3, "Categories")
                    .with_child(MindNode::new(8, "Numbers"))
                    .with_child(MindNode::new(9, "Graphics"))
            )
            .with_child(MindNode::new(4, "Patterns"))
    }

    #[test]
    fn node_find_by_id() {
        let tree = sample_tree();
        assert_eq!(tree.find(9).unwrap().label, "Graphics");
        assert!(tree.find(999).is_none());
    }

    #[test]
    fn total_count_includes_all_descendants() {
        let tree = sample_tree();
        // 1 root + 3 L1 + 3 + 2 L2 = 9
        assert_eq!(tree.total_count(), 9);
    }

    #[test]
    fn visible_count_respects_collapse() {
        let mut tree = sample_tree();
        assert_eq!(tree.visible_count(), 9);
        tree.toggle_collapse(2);   // hide Languages subtree
        // 1 root + Languages (collapsed, counted as 1) + Categories + 2 + Patterns
        // = 1 + 1 + 1 + 2 + 1 = 6
        assert_eq!(tree.visible_count(), 6);
    }

    #[test]
    fn max_depth_is_deepest_path() {
        let tree = sample_tree();
        assert_eq!(tree.max_depth(), 2);
    }

    #[test]
    fn toggle_collapse_flips_state() {
        let mut tree = sample_tree();
        assert!(!tree.find(2).unwrap().collapsed);
        assert_eq!(tree.toggle_collapse(2), Some(true));
        assert!(tree.find(2).unwrap().collapsed);
        assert_eq!(tree.toggle_collapse(2), Some(false));
    }

    #[test]
    fn toggle_collapse_missing_returns_none() {
        let mut tree = sample_tree();
        assert_eq!(tree.toggle_collapse(999), None);
    }

    #[test]
    fn radial_layout_has_root_at_centre() {
        let tree = sample_tree();
        let nodes = radial_layout(&tree, 500.0, 500.0, 100.0);
        let root = nodes.iter().find(|n| n.id == 1).unwrap();
        assert_eq!(root.x, 500.0);
        assert_eq!(root.y, 500.0);
        assert_eq!(root.depth, 0);
    }

    #[test]
    fn radial_layout_children_on_first_ring() {
        let tree = sample_tree();
        let nodes = radial_layout(&tree, 0.0, 0.0, 100.0);
        for child_id in [2, 3, 4] {
            let n = nodes.iter().find(|x| x.id == child_id).unwrap();
            let r = (n.x * n.x + n.y * n.y).sqrt();
            assert!((r - 100.0).abs() < 0.01);
            assert_eq!(n.depth, 1);
        }
    }

    #[test]
    fn radial_layout_grandchildren_on_second_ring() {
        let tree = sample_tree();
        let nodes = radial_layout(&tree, 0.0, 0.0, 100.0);
        for child_id in [5, 6, 7, 8, 9] {
            let n = nodes.iter().find(|x| x.id == child_id).unwrap();
            assert_eq!(n.depth, 2);
        }
    }

    #[test]
    fn layout_excludes_collapsed_subtree() {
        let mut tree = sample_tree();
        tree.toggle_collapse(2);
        let nodes = radial_layout(&tree, 0.0, 0.0, 100.0);
        // Collapsed Languages: still present; its children 5/6/7 are NOT.
        assert!(nodes.iter().any(|n| n.id == 2));
        assert!(!nodes.iter().any(|n| n.id == 5));
    }

    #[test]
    fn traverse_dfs_pre_order() {
        let tree = sample_tree();
        let order = traverse_dfs(&tree);
        // 1 → 2 → 5 → 6 → 7 → 3 → 8 → 9 → 4
        assert_eq!(order, vec![1, 2, 5, 6, 7, 3, 8, 9, 4]);
    }

    #[test]
    fn traverse_bfs_level_order() {
        let tree = sample_tree();
        let order = traverse_bfs(&tree);
        // 1 → 2 3 4 → 5 6 7 8 9
        assert_eq!(order, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn traversal_respects_collapse() {
        let mut tree = sample_tree();
        tree.toggle_collapse(2);
        let dfs = traverse_dfs(&tree);
        assert!(!dfs.contains(&5));
        assert!(!dfs.contains(&6));
        assert!(dfs.contains(&2));
    }

    #[test]
    fn svg_has_root_circle() {
        let tree = sample_tree();
        let nodes = radial_layout(&tree, 500.0, 500.0, 100.0);
        let svg = to_svg(&nodes, 1000.0, 1000.0, 20.0);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<circle"));
        assert!(svg.contains("Rosetta Stone"));
    }

    #[test]
    fn svg_has_edges() {
        let tree = sample_tree();
        let nodes = radial_layout(&tree, 500.0, 500.0, 100.0);
        let svg = to_svg(&nodes, 1000.0, 1000.0, 20.0);
        // Edge count = nodes.len() - 1 (tree has n-1 edges)
        let line_count = svg.matches("<line").count();
        assert_eq!(line_count, nodes.len() - 1);
    }

    #[test]
    fn svg_escapes_xml_special_chars() {
        let tree = MindNode::new(1, "A & B <tag>");
        let nodes = radial_layout(&tree, 0.0, 0.0, 100.0);
        let svg = to_svg(&nodes, 100.0, 100.0, 10.0);
        assert!(svg.contains("A &amp; B &lt;tag&gt;"));
    }

    #[test]
    fn custom_color_preserved_in_layout() {
        let tree = MindNode::new(1, "root").with_color("#ff0000");
        let nodes = radial_layout(&tree, 0.0, 0.0, 100.0);
        assert_eq!(nodes[0].color, "#ff0000");
    }

    #[test]
    fn empty_tree_single_node_layout() {
        let tree = MindNode::new(1, "alone");
        let nodes = radial_layout(&tree, 50.0, 50.0, 100.0);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].x, 50.0);
    }

    #[test]
    fn parent_id_set_correctly() {
        let tree = sample_tree();
        let nodes = radial_layout(&tree, 0.0, 0.0, 100.0);
        let n5 = nodes.iter().find(|n| n.id == 5).unwrap();
        assert_eq!(n5.parent_id, Some(2));
        let root = nodes.iter().find(|n| n.id == 1).unwrap();
        assert_eq!(root.parent_id, None);
    }
}
