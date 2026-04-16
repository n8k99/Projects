# Dijkstra's Algorithm

A choreography that finds the shortest path through a weighted graph — the optimal route from intention to fulfillment.

## Design feedback

Dijkstra's algorithm is the capstone of the Numbers category, and it reveals what the entire sequence has been building toward. A dependency graph IS a choreography. The resolver already walks graphs when it resolves nested `@references`. `@a{x: @b{y: @c}}` is a DAG with edges a->b->c. Dijkstra formalizes what the resolver does informally: find the optimal path through a graph of dependencies.

But the resolver walks the ENTIRE dependency graph — it resolves everything reachable. Dijkstra finds the SHORTEST path — it skips expensive alternatives. This suggests an optimization the resolver could make: when multiple resolution paths exist for a reference, choose the cheapest one. The resolver becomes a shortest-path finder through the space of possible resolutions.

Weighted edges in InnateScript are COSTS — time, money, computational resources. A choreography where agent A can produce a result in 2 seconds or agent B can produce it in 10 seconds has a weighted choice. Dijkstra's algorithm over the agent graph finds the cheapest choreography.

```dpn
@dijkstra{
    graph: {
        "A": [["B", 4], ["C", 2]],
        "B": [["D", 3], ["E", 1]],
        "C": [["B", 1], ["D", 5]],
        "D": [["E", 2], ["F", 6]],
        "E": [["F", 3]]
    },
    start: "A",
    goal: "F"
} -> {distance: 7, path: ["A", "C", "B", "E", "F"]}
```

The `@dijkstra` native takes a graph, a start, and a goal. It returns both the distance and the path — not just the answer but the explanation of how it got there. This is important for choreographies: the orchestrator needs to know not just the cost but the sequence of agents that will execute.

This connects back to G013's ascending-cost gates and G014's rule application. The `<-` gate pipeline IS a shortest-path problem: find the verification sequence that rejects bad inputs at minimum cost. Cheap filters first (regex check: 0.001s), expensive filters later (database lookup: 0.5s, human review: 300s). The pipeline's ordering is Dijkstra applied to verification cost.

```dpn
# Shortest path through a resolution graph
@resolve{
    target: @final_report,
    strategy: "dijkstra"
}
```

When the resolver encounters a reference with multiple resolution paths, it builds a graph. Each path has a cost — the sum of computation time, network latency, and resource consumption for each step. `strategy: "dijkstra"` tells the resolver to find the cheapest path rather than the first path or all paths.

```dpn
# All shortest distances from a source
@dijkstra_distances{
    graph: @network_topology,
    start: "gateway"
} -> {
    "gateway": 0,
    "auth_service": 2,
    "data_store": 5,
    "cache": 1,
    "renderer": 7
}
```

The distance map is a routing table. Every agent in the noosphere can compute its distance to every other agent. This is how the resolver knows the cost of any resolution path before committing to it.

## Choreographic case

Optimal routing through a multi-agent supply chain where each agent has different costs and latencies. The choreography's `where` scores the total path cost.

```dpn
# Supply chain routing: find cheapest path from raw material to delivery
@supply_chain{
    nodes: ["supplier", "manufacturer", "warehouse", "distributor", "retailer"],
    edges: [
        {from: "supplier", to: "manufacturer", cost: 50, latency: "2d"},
        {from: "supplier", to: "warehouse", cost: 80, latency: "1d"},
        {from: "manufacturer", to: "warehouse", cost: 20, latency: "3d"},
        {from: "manufacturer", to: "distributor", cost: 60, latency: "1d"},
        {from: "warehouse", to: "distributor", cost: 15, latency: "2d"},
        {from: "warehouse", to: "retailer", cost: 45, latency: "4d"},
        {from: "distributor", to: "retailer", cost: 25, latency: "1d"}
    ]
}

@dijkstra{graph: @supply_chain, start: "supplier", goal: "retailer"}
-> {cost: 110, path: ["supplier", "manufacturer", "warehouse", "distributor", "retailer"]}

where {
    @path.total_cost < 150,
    @path.each{agent -> agent.available && agent.capacity > @order.quantity}
}
```

The graph is the supply chain. The edges are contracts between agents with known costs and latencies. Dijkstra finds the cheapest route from raw material to retail. But the `where` clause adds constraints: the total cost must be under budget, and every agent along the path must have capacity. If the cheapest path violates a constraint, the resolver backtracks and finds the next cheapest — Dijkstra with pruning.

This extends to dynamic rerouting:

```dpn
# Dynamic rerouting when an agent becomes unavailable
@monitor{
    path: @current_route,
    on_failure: {agent ->
        @dijkstra{
            graph: @supply_chain.remove_node(agent),
            start: @current_position,
            goal: "retailer"
        } -> @new_route
        @reroute{from: @current_route, to: @new_route}
    }
}
```

When an agent in the path fails, the choreography rebuilds the graph without that agent and re-runs Dijkstra from the current position. The noosphere heals itself by finding the next-best path. The cost of resilience is the difference between the original shortest path and the rerouted one.

## What this means

Dijkstra's algorithm reveals the graph underneath everything InnateScript does. The graph IS the noosphere. Agents are nodes. Communication channels are edges. Weights are latency, cost, trust. Finding the shortest path from intention to fulfillment is what InnateScript does. Dijkstra is the algorithm underneath.

### META-OBSERVATION: The Numbers category arc

G001 through G015 traversed from pure math (PI) through evaluation, scheduling, world-state, trust, policy, and now graph optimization. The Numbers category didn't just implement 15 algorithms — it discovered the resolver's architecture:

- **G001 PI / G002 Fibonacci**: The computation engine. Pure functions, memoization, infinite precision.
- **G003 Calculator / G004 Binary**: The evaluation engine. Parsing, representation, base conversion.
- **G005 Mortgage / G006 Tax / G007 Tile Cost**: The domain modeler. Real-world quantities with rules and constraints.
- **G008 Unit Converter**: The context bridge. Translation between frames of reference.
- **G009 Change Making**: The optimizer. Greedy vs. dynamic programming. Finding the minimum.
- **G010 Alarm Clock / G011 Next Prime**: The scheduler. Time, sequence, "what comes next."
- **G012 Distance Cities**: The fact store. World-state, database queries, shared mutable data.
- **G013 Credit Card**: The gate pipeline. Ascending-cost validation. Reject cheaply, verify expensively.
- **G014 Prime Factors**: The rule engine. Decomposition, canonical forms, repeated application of rules.
- **G015 Dijkstra**: The graph optimizer. Shortest paths, optimal routing, the algorithm underneath the resolver itself.

Each project was a lens that revealed a different facet of what InnateScript's resolver needs to be: a computation engine + evaluation engine + domain modeler + context bridge + optimizer + scheduler + fact store + gate pipeline + rule engine + graph optimizer. The Numbers category is the resolver's blueprint, discovered one algorithm at a time.

The noosphere is a weighted graph of agents, and the resolver is Dijkstra running over it, finding the cheapest path from question to answer, from intention to fulfillment, from raw dependency to resolved value.
