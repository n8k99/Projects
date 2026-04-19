//! Family Tree Creator — the first self-referential graph with cycle prevention.
//!
//! A Person refers to other Persons (as parents). This is the first project in
//! Classes where a type is recursive through references: Person's most important
//! field is a list of PersonIds pointing back into the same collection.
//!
//! The structure is a DAG, not a tree — "family tree" is a misnomer. A person
//! has at most two parents, but join two unrelated branches of a large family
//! and you quickly get multiple paths to shared ancestors. The code treats it
//! as a graph throughout.
//!
//! Adding parents runs a **transitive closure check** to prevent cycles: the
//! candidate parent must not already be a descendant of the child. This is
//! the first project in Classes where an invariant must be enforced across the
//! entire reachable structure rather than only the directly-touched entities.

use std::collections::{HashMap, HashSet, VecDeque};

pub type PersonId = u64;

#[derive(Debug, Clone)]
pub struct Person {
    pub id: PersonId,
    pub name: String,
    pub birth: Option<i32>,
    pub death: Option<i32>,
    pub parents: Vec<PersonId>,
}

#[derive(Debug)]
pub enum FamilyError {
    UnknownPerson(PersonId),
    TooManyParents,
    DuplicateParent,
    SelfParent,
    WouldCreateCycle { child: PersonId, ancestor: PersonId },
}

pub struct FamilyTree {
    persons: HashMap<PersonId, Person>,
    children: HashMap<PersonId, HashSet<PersonId>>,
    next_id: u64,
}

impl FamilyTree {
    pub fn new() -> Self {
        Self { persons: HashMap::new(), children: HashMap::new(), next_id: 1 }
    }

    pub fn add_person(&mut self, name: &str, birth: Option<i32>, death: Option<i32>) -> PersonId {
        let id = self.next_id;
        self.next_id += 1;
        self.persons.insert(id, Person {
            id, name: name.into(), birth, death, parents: Vec::new(),
        });
        self.children.insert(id, HashSet::new());
        id
    }

    pub fn get(&self, id: PersonId) -> Option<&Person> { self.persons.get(&id) }

    /// Replace the parents of `child` with `parents`. At most two distinct parents,
    /// each must exist, none may be `child`, and none may already be a descendant
    /// of `child`.
    pub fn set_parents(&mut self, child: PersonId, parents: &[PersonId]) -> Result<(), FamilyError> {
        if !self.persons.contains_key(&child) {
            return Err(FamilyError::UnknownPerson(child));
        }
        if parents.len() > 2 { return Err(FamilyError::TooManyParents); }
        let unique: HashSet<PersonId> = parents.iter().copied().collect();
        if unique.len() != parents.len() { return Err(FamilyError::DuplicateParent); }
        for &p in parents {
            if p == child { return Err(FamilyError::SelfParent); }
            if !self.persons.contains_key(&p) {
                return Err(FamilyError::UnknownPerson(p));
            }
        }
        // Cycle check: a candidate parent must not be a descendant of child.
        let descendants = self.descendants(child, None);
        for &p in parents {
            if descendants.contains(&p) {
                return Err(FamilyError::WouldCreateCycle { child, ancestor: p });
            }
        }

        // Remove child from the children-lists of its previous parents.
        let old_parents: Vec<PersonId> = self.persons[&child].parents.clone();
        for op in old_parents {
            if let Some(set) = self.children.get_mut(&op) { set.remove(&child); }
        }
        // Install new parent links and update reverse index.
        self.persons.get_mut(&child).unwrap().parents = parents.to_vec();
        for &p in parents {
            self.children.entry(p).or_default().insert(child);
        }
        Ok(())
    }

    /// Walk parents breadth-first up to `max_depth` generations (None = unbounded).
    /// Excludes the starting person.
    pub fn ancestors(&self, id: PersonId, max_depth: Option<usize>) -> HashSet<PersonId> {
        let mut seen = HashSet::new();
        let mut queue: VecDeque<(PersonId, usize)> = VecDeque::new();
        queue.push_back((id, 0));
        while let Some((cur, depth)) = queue.pop_front() {
            if let Some(max) = max_depth { if depth >= max { continue; } }
            if let Some(p) = self.persons.get(&cur) {
                for &parent in &p.parents {
                    if seen.insert(parent) {
                        queue.push_back((parent, depth + 1));
                    }
                }
            }
        }
        seen
    }

    /// Walk children breadth-first up to `max_depth` generations. Excludes self.
    pub fn descendants(&self, id: PersonId, max_depth: Option<usize>) -> HashSet<PersonId> {
        let mut seen = HashSet::new();
        let mut queue: VecDeque<(PersonId, usize)> = VecDeque::new();
        queue.push_back((id, 0));
        while let Some((cur, depth)) = queue.pop_front() {
            if let Some(max) = max_depth { if depth >= max { continue; } }
            if let Some(kids) = self.children.get(&cur) {
                for &child in kids {
                    if seen.insert(child) {
                        queue.push_back((child, depth + 1));
                    }
                }
            }
        }
        seen
    }

    /// Set of persons who are ancestors of BOTH a and b.
    pub fn common_ancestors(&self, a: PersonId, b: PersonId) -> HashSet<PersonId> {
        let aa = self.ancestors(a, None);
        let bb = self.ancestors(b, None);
        aa.intersection(&bb).copied().collect()
    }

    /// Other persons who share at least one parent with `id` (full or half siblings).
    pub fn siblings(&self, id: PersonId) -> HashSet<PersonId> {
        let mut sibs = HashSet::new();
        let Some(p) = self.persons.get(&id) else { return sibs };
        for &parent in &p.parents {
            if let Some(kids) = self.children.get(&parent) {
                for &k in kids { if k != id { sibs.insert(k); } }
            }
        }
        sibs
    }
}

impl Default for FamilyTree {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn three_gen() -> (FamilyTree, [PersonId; 7]) {
        //   alice --- bob            carol --- dave
        //        \   /                   \    /
        //         eve                   frank
        //             \                /
        //                   gina
        let mut t = FamilyTree::new();
        let alice = t.add_person("Alice", Some(1930), None);
        let bob   = t.add_person("Bob",   Some(1928), None);
        let carol = t.add_person("Carol", Some(1935), None);
        let dave  = t.add_person("Dave",  Some(1933), None);
        let eve   = t.add_person("Eve",   Some(1955), None);
        let frank = t.add_person("Frank", Some(1960), None);
        let gina  = t.add_person("Gina",  Some(1985), None);
        t.set_parents(eve,   &[alice, bob]).unwrap();
        t.set_parents(frank, &[carol, dave]).unwrap();
        t.set_parents(gina,  &[eve, frank]).unwrap();
        (t, [alice, bob, carol, dave, eve, frank, gina])
    }

    #[test]
    fn ancestors_reach_grandparents() {
        let (t, [alice, bob, carol, dave, eve, frank, gina]) = three_gen();
        let gina_anc = t.ancestors(gina, None);
        assert_eq!(gina_anc, [alice, bob, carol, dave, eve, frank].into());

        let gina_parents_only = t.ancestors(gina, Some(1));
        assert_eq!(gina_parents_only, [eve, frank].into());
    }

    #[test]
    fn descendants_reach_grandchildren() {
        let (t, [alice, _bob, _carol, _dave, eve, _frank, gina]) = three_gen();
        let alice_desc = t.descendants(alice, None);
        assert_eq!(alice_desc, [eve, gina].into());

        let alice_kids_only = t.descendants(alice, Some(1));
        assert_eq!(alice_kids_only, [eve].into());
    }

    #[test]
    fn half_siblings_share_one_parent() {
        let mut t = FamilyTree::new();
        let mom = t.add_person("Mom", None, None);
        let dad1 = t.add_person("Dad1", None, None);
        let dad2 = t.add_person("Dad2", None, None);
        let a = t.add_person("A", None, None);
        let b = t.add_person("B", None, None);
        t.set_parents(a, &[mom, dad1]).unwrap();
        t.set_parents(b, &[mom, dad2]).unwrap();
        assert_eq!(t.siblings(a), [b].into());
        assert_eq!(t.siblings(b), [a].into());
    }

    #[test]
    fn common_ancestors_works_across_branches() {
        let (t, [alice, bob, _carol, _dave, eve, frank, gina]) = three_gen();
        // Eve and Gina share Alice and Bob as common ancestors (Eve's parents,
        // Gina's grandparents). Frank does not appear — he is Gina's but not Eve's.
        let common = t.common_ancestors(eve, gina);
        assert_eq!(common, [alice, bob].into());
        // Two unrelated persons have no common ancestors.
        assert!(t.common_ancestors(eve, frank).is_empty());
    }

    #[test]
    fn cycle_prevention_refuses_descendant_as_parent() {
        let (mut t, [alice, _bob, _c, _d, eve, _f, gina]) = three_gen();
        // Trying to make Gina (a descendant) into Alice's parent must fail.
        let err = t.set_parents(alice, &[gina]);
        assert!(matches!(err, Err(FamilyError::WouldCreateCycle { .. })));
        // And Eve can't become Alice's parent either (also a descendant).
        let err = t.set_parents(alice, &[eve]);
        assert!(matches!(err, Err(FamilyError::WouldCreateCycle { .. })));
    }

    #[test]
    fn self_parenting_refused() {
        let mut t = FamilyTree::new();
        let p = t.add_person("P", None, None);
        assert!(matches!(t.set_parents(p, &[p]), Err(FamilyError::SelfParent)));
    }

    #[test]
    fn too_many_or_duplicate_parents_refused() {
        let mut t = FamilyTree::new();
        let a = t.add_person("A", None, None);
        let b = t.add_person("B", None, None);
        let c = t.add_person("C", None, None);
        let d = t.add_person("D", None, None);
        assert!(matches!(t.set_parents(d, &[a, b, c]), Err(FamilyError::TooManyParents)));
        assert!(matches!(t.set_parents(d, &[a, a]), Err(FamilyError::DuplicateParent)));
    }

    #[test]
    fn reassigning_parents_updates_child_index() {
        let mut t = FamilyTree::new();
        let a = t.add_person("A", None, None);
        let b = t.add_person("B", None, None);
        let c = t.add_person("C", None, None);
        t.set_parents(c, &[a]).unwrap();
        assert!(t.descendants(a, None).contains(&c));
        t.set_parents(c, &[b]).unwrap();
        // c is no longer reachable from a; must be reachable from b.
        assert!(!t.descendants(a, None).contains(&c));
        assert!(t.descendants(b, None).contains(&c));
    }
}
