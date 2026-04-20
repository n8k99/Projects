//! ERD Creator — entity-relationship diagram with cardinality + topological sort.
//!
//! Eleventh Databases project. An ERD models a database schema: entities
//! (tables) with attributes, relationships with cardinalities (1:1, 1:N,
//! N:M), and the foreign-key graph between them. G111's distinctive
//! moves:
//!   * **Cardinality as data** — relationships carry a cardinality enum.
//!   * **Dependency graph walks** — which entities reference which.
//!   * **Topological sort** for presentation order (dependencies first).
//!   * **Cycle detection** in the FK graph — flag circular references.
//!   * **Graphviz DOT export** — emit the diagram as a rendering-ready
//!      text format.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality { OneToOne, OneToMany, ManyToMany }

impl Cardinality {
    pub fn dot_arrow(&self) -> &'static str {
        match self {
            Cardinality::OneToOne => "none",
            Cardinality::OneToMany => "crow",
            Cardinality::ManyToMany => "crowodot",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub ty: String,        // simple text: "INT", "TEXT", etc.
    pub primary_key: bool,
    pub nullable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub name: String,
    pub attributes: Vec<Attribute>,
}

impl Entity {
    pub fn primary_keys(&self) -> Vec<String> {
        self.attributes.iter()
            .filter(|a| a.primary_key)
            .map(|a| a.name.clone())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relationship {
    pub from_entity: String,
    pub from_attr: String,
    pub to_entity: String,
    pub to_attr: String,
    pub cardinality: Cardinality,
}

#[derive(Debug, Clone)]
pub struct Erd {
    pub entities: BTreeMap<String, Entity>,
    pub relationships: Vec<Relationship>,
}

impl Erd {
    pub fn new() -> Self {
        Self { entities: BTreeMap::new(), relationships: Vec::new() }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(entity.name.clone(), entity);
    }

    pub fn add_relationship(&mut self, rel: Relationship) {
        self.relationships.push(rel);
    }

    /// Dependencies: which entities this entity references via FK.
    pub fn dependencies_of(&self, entity: &str) -> Vec<String> {
        let mut out: BTreeSet<String> = BTreeSet::new();
        for r in &self.relationships {
            if r.from_entity == entity && r.to_entity != entity {
                out.insert(r.to_entity.clone());
            }
        }
        out.into_iter().collect()
    }

    /// Topological sort of entities — referenced entities come first.
    /// Returns `None` if a cycle is detected.
    pub fn topo_sort(&self) -> Option<Vec<String>> {
        // Kahn's algorithm.
        let mut in_degree: BTreeMap<String, usize> = BTreeMap::new();
        for name in self.entities.keys() {
            in_degree.insert(name.clone(), 0);
        }
        for r in &self.relationships {
            if r.from_entity != r.to_entity
                && self.entities.contains_key(&r.from_entity)
                && self.entities.contains_key(&r.to_entity)
            {
                *in_degree.entry(r.from_entity.clone()).or_insert(0) += 1;
            }
        }
        let mut queue: Vec<String> = in_degree.iter()
            .filter(|(_, &d)| d == 0)
            .map(|(n, _)| n.clone())
            .collect();
        queue.sort();
        let mut out: Vec<String> = Vec::new();
        while let Some(n) = queue.pop() {
            out.push(n.clone());
            for r in &self.relationships {
                if r.to_entity == n && r.from_entity != r.to_entity {
                    if let Some(d) = in_degree.get_mut(&r.from_entity) {
                        if *d > 0 { *d -= 1; }
                        if *d == 0 {
                            if !out.contains(&r.from_entity)
                                && !queue.contains(&r.from_entity) {
                                queue.push(r.from_entity.clone());
                                queue.sort();
                            }
                        }
                    }
                }
            }
        }
        if out.len() != self.entities.len() { None } else { Some(out) }
    }

    /// Cycles in the FK graph — non-empty if circular references exist.
    pub fn find_cycles(&self) -> Vec<Vec<String>> {
        // Build adjacency.
        let mut adj: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for name in self.entities.keys() {
            adj.insert(name.clone(), vec![]);
        }
        for r in &self.relationships {
            if r.from_entity != r.to_entity
                && self.entities.contains_key(&r.from_entity)
                && self.entities.contains_key(&r.to_entity)
            {
                adj.get_mut(&r.from_entity).unwrap().push(r.to_entity.clone());
            }
        }
        let mut cycles = Vec::new();
        let mut visited = BTreeSet::new();
        for start in self.entities.keys() {
            if visited.contains(start) { continue; }
            let mut path: Vec<String> = vec![start.clone()];
            let mut on_path = BTreeSet::new();
            on_path.insert(start.clone());
            dfs_cycles(&adj, start, &mut path, &mut on_path, &mut visited, &mut cycles);
        }
        cycles
    }

    /// Relationships where `to_attr` doesn't exist on `to_entity`, etc.
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        for (i, r) in self.relationships.iter().enumerate() {
            if let Some(from) = self.entities.get(&r.from_entity) {
                if !from.attributes.iter().any(|a| a.name == r.from_attr) {
                    issues.push(format!(
                        "relationship {i}: {}.{} — attribute missing",
                        r.from_entity, r.from_attr));
                }
            } else {
                issues.push(format!("relationship {i}: unknown from_entity {}", r.from_entity));
            }
            if let Some(to) = self.entities.get(&r.to_entity) {
                if !to.attributes.iter().any(|a| a.name == r.to_attr) {
                    issues.push(format!(
                        "relationship {i}: {}.{} — attribute missing",
                        r.to_entity, r.to_attr));
                }
            } else {
                issues.push(format!("relationship {i}: unknown to_entity {}", r.to_entity));
            }
        }
        issues
    }

    /// Emit as Graphviz DOT.
    pub fn to_dot(&self) -> String {
        let mut out = String::from("digraph ERD {\n");
        out.push_str("  node [shape=record];\n");
        let sorted_names: Vec<&String> = self.entities.keys().collect();
        for name in sorted_names {
            let e = &self.entities[name];
            let attrs = e.attributes.iter()
                .map(|a| {
                    let marker = if a.primary_key { "🔑 " } else { "" };
                    format!("{marker}{}: {}", a.name, a.ty)
                })
                .collect::<Vec<_>>()
                .join("\\l");
            out.push_str(&format!("  {} [label=\"{{{}|{attrs}\\l}}\"];\n", name, name));
        }
        for r in &self.relationships {
            out.push_str(&format!(
                "  {} -> {} [label=\"{}.{} -> {}.{}\", arrowhead=\"{}\"];\n",
                r.from_entity, r.to_entity,
                r.from_entity, r.from_attr, r.to_entity, r.to_attr,
                r.cardinality.dot_arrow()));
        }
        out.push_str("}\n");
        out
    }
}

impl Default for Erd { fn default() -> Self { Self::new() } }

fn dfs_cycles(
    adj: &BTreeMap<String, Vec<String>>,
    node: &str,
    path: &mut Vec<String>,
    on_path: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    if let Some(children) = adj.get(node) {
        for child in children {
            if on_path.contains(child) {
                // Found a back-edge — extract the cycle.
                let start = path.iter().position(|n| n == child).unwrap();
                let mut cycle: Vec<String> = path[start..].to_vec();
                cycle.push(child.clone());
                cycles.push(cycle);
            } else if !visited.contains(child) {
                path.push(child.clone());
                on_path.insert(child.clone());
                dfs_cycles(adj, child, path, on_path, visited, cycles);
                path.pop();
                on_path.remove(child);
            }
        }
    }
    visited.insert(node.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attr(name: &str, ty: &str) -> Attribute {
        Attribute { name: name.into(), ty: ty.into(), primary_key: false, nullable: false }
    }
    fn pk(name: &str, ty: &str) -> Attribute {
        Attribute { name: name.into(), ty: ty.into(), primary_key: true, nullable: false }
    }

    fn sample_erd() -> Erd {
        let mut erd = Erd::new();
        erd.add_entity(Entity {
            name: "User".into(),
            attributes: vec![pk("id", "INT"), attr("email", "TEXT"), attr("name", "TEXT")],
        });
        erd.add_entity(Entity {
            name: "Order".into(),
            attributes: vec![pk("id", "INT"), attr("user_id", "INT"), attr("total", "INT")],
        });
        erd.add_entity(Entity {
            name: "OrderItem".into(),
            attributes: vec![pk("id", "INT"), attr("order_id", "INT"), attr("product_id", "INT")],
        });
        erd.add_entity(Entity {
            name: "Product".into(),
            attributes: vec![pk("id", "INT"), attr("name", "TEXT"), attr("price", "INT")],
        });
        erd.add_relationship(Relationship {
            from_entity: "Order".into(), from_attr: "user_id".into(),
            to_entity: "User".into(), to_attr: "id".into(),
            cardinality: Cardinality::OneToMany,
        });
        erd.add_relationship(Relationship {
            from_entity: "OrderItem".into(), from_attr: "order_id".into(),
            to_entity: "Order".into(), to_attr: "id".into(),
            cardinality: Cardinality::OneToMany,
        });
        erd.add_relationship(Relationship {
            from_entity: "OrderItem".into(), from_attr: "product_id".into(),
            to_entity: "Product".into(), to_attr: "id".into(),
            cardinality: Cardinality::ManyToMany,
        });
        erd
    }

    #[test]
    fn primary_keys_listed() {
        let erd = sample_erd();
        assert_eq!(erd.entities["User"].primary_keys(), vec!["id"]);
    }

    #[test]
    fn dependencies_of_order() {
        let erd = sample_erd();
        assert_eq!(erd.dependencies_of("Order"), vec!["User"]);
    }

    #[test]
    fn dependencies_of_order_item() {
        let erd = sample_erd();
        let deps = erd.dependencies_of("OrderItem");
        assert!(deps.contains(&"Order".to_string()));
        assert!(deps.contains(&"Product".to_string()));
    }

    #[test]
    fn dependencies_of_user_is_empty() {
        let erd = sample_erd();
        assert!(erd.dependencies_of("User").is_empty());
    }

    #[test]
    fn topo_sort_respects_dependencies() {
        let erd = sample_erd();
        let sort = erd.topo_sort().unwrap();
        let user_idx = sort.iter().position(|n| n == "User").unwrap();
        let order_idx = sort.iter().position(|n| n == "Order").unwrap();
        let product_idx = sort.iter().position(|n| n == "Product").unwrap();
        let item_idx = sort.iter().position(|n| n == "OrderItem").unwrap();
        assert!(user_idx < order_idx);
        assert!(order_idx < item_idx);
        assert!(product_idx < item_idx);
    }

    #[test]
    fn no_cycles_in_sample_erd() {
        let erd = sample_erd();
        assert!(erd.find_cycles().is_empty());
    }

    #[test]
    fn circular_references_detected() {
        let mut erd = Erd::new();
        erd.add_entity(Entity { name: "A".into(), attributes: vec![pk("id", "INT"), attr("b_id", "INT")] });
        erd.add_entity(Entity { name: "B".into(), attributes: vec![pk("id", "INT"), attr("a_id", "INT")] });
        erd.add_relationship(Relationship {
            from_entity: "A".into(), from_attr: "b_id".into(),
            to_entity: "B".into(), to_attr: "id".into(),
            cardinality: Cardinality::OneToOne,
        });
        erd.add_relationship(Relationship {
            from_entity: "B".into(), from_attr: "a_id".into(),
            to_entity: "A".into(), to_attr: "id".into(),
            cardinality: Cardinality::OneToOne,
        });
        assert!(!erd.find_cycles().is_empty());
        assert!(erd.topo_sort().is_none());
    }

    #[test]
    fn validate_catches_missing_attribute() {
        let mut erd = sample_erd();
        erd.add_relationship(Relationship {
            from_entity: "Order".into(), from_attr: "nonexistent".into(),
            to_entity: "User".into(), to_attr: "id".into(),
            cardinality: Cardinality::OneToOne,
        });
        let issues = erd.validate();
        assert!(issues.iter().any(|s| s.contains("nonexistent")));
    }

    #[test]
    fn validate_catches_unknown_entity() {
        let mut erd = sample_erd();
        erd.add_relationship(Relationship {
            from_entity: "Ghost".into(), from_attr: "x".into(),
            to_entity: "User".into(), to_attr: "id".into(),
            cardinality: Cardinality::OneToOne,
        });
        let issues = erd.validate();
        assert!(issues.iter().any(|s| s.contains("Ghost")));
    }

    #[test]
    fn validate_clean_erd_has_no_issues() {
        let erd = sample_erd();
        assert!(erd.validate().is_empty());
    }

    #[test]
    fn dot_export_includes_entity_nodes() {
        let erd = sample_erd();
        let dot = erd.to_dot();
        assert!(dot.contains("digraph ERD"));
        assert!(dot.contains("User"));
        assert!(dot.contains("Order"));
    }

    #[test]
    fn dot_export_includes_relationship_edges() {
        let erd = sample_erd();
        let dot = erd.to_dot();
        assert!(dot.contains("Order -> User"));
        assert!(dot.contains("OrderItem -> Product"));
    }

    #[test]
    fn topo_sort_of_empty_erd_is_empty() {
        let erd = Erd::new();
        assert!(erd.topo_sort().unwrap().is_empty());
    }

    #[test]
    fn self_referencing_entity_is_not_a_cycle() {
        let mut erd = Erd::new();
        erd.add_entity(Entity {
            name: "Node".into(),
            attributes: vec![pk("id", "INT"), attr("parent_id", "INT")],
        });
        erd.add_relationship(Relationship {
            from_entity: "Node".into(), from_attr: "parent_id".into(),
            to_entity: "Node".into(), to_attr: "id".into(),
            cardinality: Cardinality::OneToMany,
        });
        // Self-ref is skipped in dependency tracking.
        assert!(erd.dependencies_of("Node").is_empty());
        // Topo sort should still succeed.
        assert!(erd.topo_sort().is_some());
    }
}
