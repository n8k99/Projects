"""ERD Creator — entity-relationship diagram with cardinality + topological sort.

Entities hold Attributes; Relationships carry Cardinality.
topo_sort returns None on cycle. find_cycles walks FK graph via DFS.
DOT export yields Graphviz-compatible text.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class Cardinality(Enum):
    ONE_TO_ONE = "one_to_one"
    ONE_TO_MANY = "one_to_many"
    MANY_TO_MANY = "many_to_many"

    def dot_arrow(self) -> str:
        return {
            Cardinality.ONE_TO_ONE: "none",
            Cardinality.ONE_TO_MANY: "crow",
            Cardinality.MANY_TO_MANY: "crowodot",
        }[self]


@dataclass
class Attribute:
    name: str
    ty: str
    primary_key: bool = False
    nullable: bool = False


@dataclass
class Entity:
    name: str
    attributes: list[Attribute] = field(default_factory=list)

    def primary_keys(self) -> list[str]:
        return [a.name for a in self.attributes if a.primary_key]


@dataclass
class Relationship:
    from_entity: str
    from_attr: str
    to_entity: str
    to_attr: str
    cardinality: Cardinality


@dataclass
class Erd:
    entities: dict[str, Entity] = field(default_factory=dict)
    relationships: list[Relationship] = field(default_factory=list)

    def add_entity(self, e: Entity) -> None:
        self.entities[e.name] = e

    def add_relationship(self, r: Relationship) -> None:
        self.relationships.append(r)

    def dependencies_of(self, entity: str) -> list[str]:
        deps = {r.to_entity for r in self.relationships
                if r.from_entity == entity and r.to_entity != entity}
        return sorted(deps)

    def topo_sort(self) -> list[str] | None:
        in_degree: dict[str, int] = {name: 0 for name in self.entities}
        for r in self.relationships:
            if r.from_entity != r.to_entity \
               and r.from_entity in self.entities \
               and r.to_entity in self.entities:
                in_degree[r.from_entity] = in_degree.get(r.from_entity, 0) + 1
        queue = sorted(n for n, d in in_degree.items() if d == 0)
        out: list[str] = []
        while queue:
            n = queue.pop(0)
            out.append(n)
            for r in self.relationships:
                if r.to_entity == n and r.from_entity != r.to_entity:
                    if r.from_entity in in_degree and in_degree[r.from_entity] > 0:
                        in_degree[r.from_entity] -= 1
                        if in_degree[r.from_entity] == 0 \
                           and r.from_entity not in out \
                           and r.from_entity not in queue:
                            queue.append(r.from_entity)
                            queue.sort()
        if len(out) != len(self.entities):
            return None
        return out

    def find_cycles(self) -> list[list[str]]:
        adj: dict[str, list[str]] = {name: [] for name in self.entities}
        for r in self.relationships:
            if r.from_entity != r.to_entity \
               and r.from_entity in self.entities \
               and r.to_entity in self.entities:
                adj[r.from_entity].append(r.to_entity)

        cycles: list[list[str]] = []
        visited: set[str] = set()

        def dfs(node: str, path: list[str], on_path: set[str]):
            for child in adj.get(node, []):
                if child in on_path:
                    start = path.index(child)
                    cycles.append(path[start:] + [child])
                elif child not in visited:
                    path.append(child)
                    on_path.add(child)
                    dfs(child, path, on_path)
                    path.pop()
                    on_path.discard(child)
            visited.add(node)

        for start in sorted(self.entities):
            if start in visited:
                continue
            dfs(start, [start], {start})
        return cycles

    def validate(self) -> list[str]:
        issues = []
        for i, r in enumerate(self.relationships):
            from_e = self.entities.get(r.from_entity)
            to_e = self.entities.get(r.to_entity)
            if from_e is None:
                issues.append(f"relationship {i}: unknown from_entity {r.from_entity}")
            elif not any(a.name == r.from_attr for a in from_e.attributes):
                issues.append(f"relationship {i}: {r.from_entity}.{r.from_attr} — attribute missing")
            if to_e is None:
                issues.append(f"relationship {i}: unknown to_entity {r.to_entity}")
            elif not any(a.name == r.to_attr for a in to_e.attributes):
                issues.append(f"relationship {i}: {r.to_entity}.{r.to_attr} — attribute missing")
        return issues

    def to_dot(self) -> str:
        lines = ["digraph ERD {", "  node [shape=record];"]
        for name in sorted(self.entities):
            e = self.entities[name]
            attrs = []
            for a in e.attributes:
                marker = "🔑 " if a.primary_key else ""
                attrs.append(f"{marker}{a.name}: {a.ty}")
            attrs_str = r"\l".join(attrs)
            lines.append(f'  {name} [label="{{{name}|{attrs_str}\\l}}"];')
        for r in self.relationships:
            lines.append(
                f'  {r.from_entity} -> {r.to_entity} '
                f'[label="{r.from_entity}.{r.from_attr} -> {r.to_entity}.{r.to_attr}", '
                f'arrowhead="{r.cardinality.dot_arrow()}"];'
            )
        lines.append("}")
        return "\n".join(lines) + "\n"


def demo() -> None:
    erd = Erd()
    erd.add_entity(Entity("User", [
        Attribute("id", "INT", primary_key=True),
        Attribute("email", "TEXT"),
        Attribute("name", "TEXT"),
    ]))
    erd.add_entity(Entity("Order", [
        Attribute("id", "INT", primary_key=True),
        Attribute("user_id", "INT"),
        Attribute("total", "INT"),
    ]))
    erd.add_entity(Entity("OrderItem", [
        Attribute("id", "INT", primary_key=True),
        Attribute("order_id", "INT"),
        Attribute("product_id", "INT"),
    ]))
    erd.add_entity(Entity("Product", [
        Attribute("id", "INT", primary_key=True),
        Attribute("name", "TEXT"),
        Attribute("price", "INT"),
    ]))
    erd.add_relationship(Relationship("Order", "user_id", "User", "id", Cardinality.ONE_TO_MANY))
    erd.add_relationship(Relationship("OrderItem", "order_id", "Order", "id", Cardinality.ONE_TO_MANY))
    erd.add_relationship(Relationship("OrderItem", "product_id", "Product", "id", Cardinality.MANY_TO_MANY))

    print("=== topological sort ===")
    print(f"  {erd.topo_sort()}")
    print("\n=== cycles ===")
    print(f"  {erd.find_cycles()}")
    print("\n=== validation ===")
    for issue in erd.validate():
        print(f"  {issue}")
    print("\n=== DOT export (first 200 chars) ===")
    print(erd.to_dot()[:200])


if __name__ == "__main__":
    demo()
