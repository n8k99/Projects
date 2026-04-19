"""Family Tree Creator — the first self-referential graph with cycle prevention.

A ``Person`` refers to other persons (as parents). This is the first project
in Classes where the type is recursive through references: ``Person``'s most
important field is a list of ids pointing back into the same collection.

The structure is a DAG, not a tree — "family tree" is a misnomer. A person
has at most two parents, but joining two unrelated branches of a large family
gives multiple paths to shared ancestors. The code treats it as a graph.

Adding parents runs a **transitive closure check** to prevent cycles: the
candidate parent must not already be a descendant of the child. This is the
first project in Classes where an invariant is enforced across the entire
reachable structure rather than only the directly-touched entities.
"""

from __future__ import annotations
from collections import deque
from dataclasses import dataclass, field


PersonId = int


@dataclass
class Person:
    id: PersonId
    name: str
    birth: int | None = None
    death: int | None = None
    parents: list[PersonId] = field(default_factory=list)


class FamilyError(Exception): ...
class UnknownPerson(FamilyError): ...
class TooManyParents(FamilyError): ...
class DuplicateParent(FamilyError): ...
class SelfParent(FamilyError): ...
class WouldCreateCycle(FamilyError): ...


class FamilyTree:
    def __init__(self) -> None:
        self._persons: dict[PersonId, Person] = {}
        self._children: dict[PersonId, set[PersonId]] = {}
        self._next_id = 1

    def add_person(self, name: str, birth: int | None = None,
                   death: int | None = None) -> PersonId:
        pid = self._next_id
        self._next_id += 1
        self._persons[pid] = Person(id=pid, name=name, birth=birth, death=death)
        self._children[pid] = set()
        return pid

    def get(self, pid: PersonId) -> Person:
        if pid not in self._persons:
            raise UnknownPerson(f"unknown person: {pid}")
        return self._persons[pid]

    def set_parents(self, child: PersonId, parents: list[PersonId]) -> None:
        """Replace the parents of `child` with `parents` (0, 1, or 2 distinct ids).

        Raises if any parent is unknown, duplicated, equal to child, or would
        create a cycle (i.e., is already a descendant of child).
        """
        if child not in self._persons:
            raise UnknownPerson(f"unknown person: {child}")
        if len(parents) > 2:
            raise TooManyParents(f"at most 2 parents, got {len(parents)}")
        if len(set(parents)) != len(parents):
            raise DuplicateParent("parents list contains duplicates")
        for p in parents:
            if p == child:
                raise SelfParent("a person cannot be their own parent")
            if p not in self._persons:
                raise UnknownPerson(f"unknown person: {p}")
        descendants = self.descendants(child)
        for p in parents:
            if p in descendants:
                raise WouldCreateCycle(
                    f"{p} is already a descendant of {child}; "
                    "parent link would create a cycle")

        # Remove child from old parents' children sets.
        for op in self._persons[child].parents:
            self._children[op].discard(child)
        # Install new links.
        self._persons[child].parents = list(parents)
        for p in parents:
            self._children[p].add(child)

    def ancestors(self, pid: PersonId, max_depth: int | None = None) -> set[PersonId]:
        """BFS over parents, up to `max_depth` generations (None = unbounded).
        Does not include `pid` itself."""
        seen: set[PersonId] = set()
        queue: deque[tuple[PersonId, int]] = deque([(pid, 0)])
        while queue:
            cur, depth = queue.popleft()
            if max_depth is not None and depth >= max_depth:
                continue
            for parent in self._persons[cur].parents if cur in self._persons else ():
                if parent not in seen:
                    seen.add(parent)
                    queue.append((parent, depth + 1))
        return seen

    def descendants(self, pid: PersonId, max_depth: int | None = None) -> set[PersonId]:
        """BFS over children, up to `max_depth` generations. Does not include `pid`."""
        seen: set[PersonId] = set()
        queue: deque[tuple[PersonId, int]] = deque([(pid, 0)])
        while queue:
            cur, depth = queue.popleft()
            if max_depth is not None and depth >= max_depth:
                continue
            for child in self._children.get(cur, set()):
                if child not in seen:
                    seen.add(child)
                    queue.append((child, depth + 1))
        return seen

    def common_ancestors(self, a: PersonId, b: PersonId) -> set[PersonId]:
        """Persons who are ancestors of both a and b (set intersection)."""
        return self.ancestors(a) & self.ancestors(b)

    def siblings(self, pid: PersonId) -> set[PersonId]:
        """Persons sharing at least one parent with `pid` (full and half siblings)."""
        sibs: set[PersonId] = set()
        if pid not in self._persons:
            return sibs
        for p in self._persons[pid].parents:
            for k in self._children.get(p, set()):
                if k != pid:
                    sibs.add(k)
        return sibs


if __name__ == "__main__":
    t = FamilyTree()
    alice = t.add_person("Alice", 1930)
    bob   = t.add_person("Bob",   1928)
    carol = t.add_person("Carol", 1935)
    dave  = t.add_person("Dave",  1933)
    eve   = t.add_person("Eve",   1955)
    frank = t.add_person("Frank", 1960)
    gina  = t.add_person("Gina",  1985)
    t.set_parents(eve,   [alice, bob])
    t.set_parents(frank, [carol, dave])
    t.set_parents(gina,  [eve, frank])

    names = {alice: "Alice", bob: "Bob", carol: "Carol", dave: "Dave",
             eve: "Eve", frank: "Frank", gina: "Gina"}

    def show(s: set[int]) -> str:
        return "{" + ", ".join(sorted(names[p] for p in s)) + "}"

    print(f"Gina's ancestors (unbounded): {show(t.ancestors(gina))}")
    print(f"Gina's parents only (depth 1): {show(t.ancestors(gina, max_depth=1))}")
    print(f"Alice's descendants: {show(t.descendants(alice))}")
    print(f"Common ancestors of Eve and Gina: {show(t.common_ancestors(eve, gina))}")
    print(f"Common ancestors of Eve and Frank (unrelated branches): "
          f"{show(t.common_ancestors(eve, frank))}")

    # Cycle prevention: making Gina a parent of Alice should fail.
    try:
        t.set_parents(alice, [gina])
    except WouldCreateCycle as e:
        print(f"refused: {e}")
