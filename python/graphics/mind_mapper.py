"""Mind Mapper — radial layout + SVG emission.

Recursive tree (root + nested children). Radial layout: depth → radius,
sibling arc split evenly. SVG emission: edges (lines) + nodes (circles
with labels). Collapsed nodes hide their subtree from layout + traversal.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from collections import deque
import math


@dataclass
class MindNode:
    id: int
    label: str
    color: str = "#3b82f6"
    collapsed: bool = False
    children: list[MindNode] = field(default_factory=list)

    def with_child(self, c: MindNode) -> MindNode:
        self.children.append(c)
        return self

    def with_color(self, color: str) -> MindNode:
        self.color = color
        return self

    def find(self, id: int) -> MindNode | None:
        if self.id == id:
            return self
        for c in self.children:
            found = c.find(id)
            if found is not None:
                return found
        return None

    def toggle_collapse(self, id: int) -> bool | None:
        n = self.find(id)
        if n is None:
            return None
        n.collapsed = not n.collapsed
        return n.collapsed

    def visible_count(self) -> int:
        if self.collapsed:
            return 1
        return 1 + sum(c.visible_count() for c in self.children)

    def total_count(self) -> int:
        return 1 + sum(c.total_count() for c in self.children)

    def max_depth(self) -> int:
        if not self.children or self.collapsed:
            return 0
        return 1 + max(c.max_depth() for c in self.children)


@dataclass
class LaidOutNode:
    id: int
    label: str
    color: str
    depth: int
    x: float
    y: float
    parent_id: int | None
    collapsed: bool


def radial_layout(root: MindNode, cx: float, cy: float,
                  radius_step: float) -> list[LaidOutNode]:
    out: list[LaidOutNode] = []
    out.append(LaidOutNode(root.id, root.label, root.color, 0,
                           cx, cy, None, root.collapsed))
    if not root.collapsed:
        _layout_children(root, 0.0, 2 * math.pi, 1, radius_step, cx, cy, out)
    return out


def _layout_children(parent: MindNode, arc_start: float, arc_end: float,
                      depth: int, radius_step: float,
                      cx: float, cy: float, out: list[LaidOutNode]) -> None:
    n = len(parent.children)
    if n == 0:
        return
    slice_ = (arc_end - arc_start) / n
    for i, child in enumerate(parent.children):
        a0 = arc_start + slice_ * i
        a1 = arc_start + slice_ * (i + 1)
        angle = (a0 + a1) / 2
        r = depth * radius_step
        x = cx + r * math.cos(angle)
        y = cy + r * math.sin(angle)
        out.append(LaidOutNode(child.id, child.label, child.color, depth,
                               x, y, parent.id, child.collapsed))
        if not child.collapsed:
            _layout_children(child, a0, a1, depth + 1, radius_step, cx, cy, out)


def traverse_dfs(root: MindNode) -> list[int]:
    out: list[int] = []
    _dfs_visit(root, out)
    return out


def _dfs_visit(node: MindNode, out: list[int]) -> None:
    out.append(node.id)
    if node.collapsed:
        return
    for c in node.children:
        _dfs_visit(c, out)


def traverse_bfs(root: MindNode) -> list[int]:
    out: list[int] = []
    queue: deque = deque([root])
    while queue:
        n = queue.popleft()
        out.append(n.id)
        if not n.collapsed:
            for c in n.children:
                queue.append(c)
    return out


def to_svg(nodes: list[LaidOutNode], width: float, height: float,
           radius: float) -> str:
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">'
    ]
    id_to_node = {n.id: n for n in nodes}
    for n in nodes:
        if n.parent_id is not None and n.parent_id in id_to_node:
            p = id_to_node[n.parent_id]
            parts.append(
                f'  <line x1="{p.x:.2f}" y1="{p.y:.2f}" '
                f'x2="{n.x:.2f}" y2="{n.y:.2f}" stroke="#cbd5e1" stroke-width="2"/>'
            )
    for n in nodes:
        parts.append(
            f'  <circle cx="{n.x:.2f}" cy="{n.y:.2f}" r="{radius}" fill="{n.color}"/>'
        )
        parts.append(
            f'  <text x="{n.x:.2f}" y="{n.y:.2f}" text-anchor="middle" dy="0.35em" fill="white" font-size="12">{_escape_xml(n.label)}</text>'
        )
    parts.append("</svg>")
    return "\n".join(parts) + "\n"


def _escape_xml(s: str) -> str:
    return (s.replace("&", "&amp;")
             .replace("<", "&lt;")
             .replace(">", "&gt;")
             .replace('"', "&quot;"))


def demo() -> None:
    tree = (
        MindNode(1, "Rosetta Stone")
        .with_child(
            MindNode(2, "Languages")
            .with_child(MindNode(5, "Rust"))
            .with_child(MindNode(6, "Python"))
        )
        .with_child(
            MindNode(3, "Categories")
            .with_child(MindNode(8, "Graphics"))
        )
    )
    nodes = radial_layout(tree, 500.0, 500.0, 150.0)
    print(f"total layout nodes: {len(nodes)}")
    for n in nodes:
        print(f"  #{n.id:2} {n.label:20} depth={n.depth} at ({n.x:6.1f}, {n.y:6.1f})")

    print(f"\nDFS: {traverse_dfs(tree)}")
    print(f"BFS: {traverse_bfs(tree)}")

    tree.toggle_collapse(2)
    print(f"\nAfter collapse(2), visible count: {tree.visible_count()}")


if __name__ == "__main__":
    demo()
