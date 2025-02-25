"""
This module implements the algorithms that find a Hamiltonian path and a Hamiltonian cycle on a tournament graph.

References:
- Hamiltonian Path in a tournament graph:  
  P. Hell and M. Rosenfeld, *The complexity of finding generalized paths in tournaments*,  
  J. Algorithms 4 (1982) 303-309.  
  [Link](https://www.sciencedirect.com/science/article/abs/pii/0196677483900111)

- Hamiltonian Cycle in a strongly connected tournament graph:  
  Y. Manoussakis, *A Linear-Time Algorithm for Finding Hamiltonian Cycles in Tournaments*,  
  Discrete Appl. Math. 36, 2 (1992), 199–201.  
  [Link](https://www.sciencedirect.com/science/article/pii/0166218X9290233Z)
"""

from typing import Set, Dict, List, Tuple
from schemas import GraphNodeId, HashValue

class DirectedGraph:
    def __init__(self):
        self.nodes: Set[GraphNodeId] = {}
        self.edges: Dict[GraphNodeId, Set[GraphNodeId]] = {}
        self.is_tournament_graph = None
        self.connected_components: List[Tuple[GraphNodeId, HashValue]] = None
        self.node_to_scc_hash: Dict[GraphNodeId, HashValue] = None
        self.hash_to_scc_nodes: Dict[HashValue, List[GraphNodeId]] = None

    def reset_graph_properties(self):
      self.is_tournament_graph = None
      self.connected_components = None
      self.node_to_scc_hash = None
      self.hash_to_scc_nodes = None

    def add_node(self, nodeId: GraphNodeId):
        if nodeId in self.nodes:
            return
        self.nodes.add(nodeId)
        self.edges[nodeId] = set()
        # reset the graph properties
        self.reset_graph_properties()
    
    def add_directed_edge(self, nodeId1: GraphNodeId, nodeId2: GraphNodeId):
        if nodeId1 not in self.nodes:
            self.add_node(nodeId1)
        self.edges[nodeId1].add(nodeId2)
        # reset the graph properties
        self.reset_graph_properties()

    def has_edge(self, nodeId1: GraphNodeId, nodeId2: GraphNodeId) -> bool:
        return nodeId1 in self.edges and nodeId2 in self.edges[nodeId1]
    
    def assert_is_tournament_graph(self):
      if self.is_tournament_graph is None:
        self.is_tournament_graph = self.is_tournament_graph()
      assert self.is_tournament_graph

    def is_tournament_graph(self) -> bool:
        for node1 in self.nodes:
            for node2 in self.nodes:
                if node1 != node2:
                  count = (1 if self.has_edge(node1, node2) else 0) + (1 if self.has_edge(node2, node1) else 0)
                  if count != 1:
                    return False
        return True
    
    """
    Find the strongly connected components of the graph and return them in the topological order of the SCCs
    """
    def find_strongly_connected_components(self) -> List[Tuple[List[GraphNodeId], HashValue]]:
        assert self.connected_components is None
        self.connected_components: List[Tuple[GraphNodeId, HashValue]] = []
        
        # tarjan's algorithm
        index = 0
        indices = {}
        lowlinks = {}
        stack = []
        components: List[Tuple[List[GraphNodeId], HashValue]] = []

        def strongconnect(nodeId: GraphNodeId) -> None:
            assert nodeId not in indices
            nonlocal index, indices, lowlinks, stack, on_stack, components
            # Set the depth index for node
            indices[nodeId] = index
            lowlinks[nodeId] = index
            index += 1
            stack.append(nodeId)

            # Consider successors of node
            for successorId in self.edges[nodeId]:
                if successorId not in indices:
                    # Successor has not yet been visited; recurse on it
                    strongconnect(successorId)
                    lowlinks[nodeId] = min(lowlinks[nodeId], lowlinks[successorId])
                else:
                    lowlinks[nodeId] = min(lowlinks[nodeId], indices[successorId])

            # If node is a root node, pop the stack and generate an SCC
            if lowlinks[nodeId] == indices[nodeId]:
                component = []
                while True:
                    vertexId = stack.pop()
                    component.append(vertexId)
                    if vertexId == nodeId:
                        break
                components.append([component, hash(tuple(sorted(component)))])

        # Find SCCs for all nodes
        for nodeId in self.nodes:
            if nodeId not in indices:
                strongconnect(nodeId)

        self.node_to_scc_hash: Dict[GraphNodeId, HashValue] = {nodeId: component[1] for component in enumerate(components) for nodeId in component[0]}
        self.hash_to_scc_nodes: Dict[HashValue, List[GraphNodeId]] = {component[1]: component[0] for component in components}
        
        self.connected_components: List[Tuple[List[GraphNodeId], HashValue]] = []

        visited_sccs = set()
        def sort_sccs(comp: Tuple[List[GraphNodeId], HashValue]):
          assert comp[1] not in visited_sccs
          self.connected_components.append(comp)
          for u in comp[0]:
            if u in self.edges:
              for v in self.edges[u]:
                if not self.node_to_scc_hash[v] in visited_sccs:
                  sort_sccs(self.hash_to_scc_nodes[self.node_to_scc_hash[v]])
          visited_sccs.add(comp[1])

        for component in components:
          if component[1] not in visited_sccs:
            sort_sccs(component)
        
        return self.connected_components

    def assert_is_strongly_connected_component(self, hamiltonian_path: List[GraphNodeId]):
      assert self.connected_components is not None
      hash_value = hash(tuple(sorted(hamiltonian_path)))
      assert hash_value in [connected_component[1] for connected_component in self.connected_components]

    def assert_is_hamiltonian_path(self, hamiltonian_path: List[GraphNodeId]):
      for i in range(1, len(hamiltonian_path)):
        assert self.has_edge(hamiltonian_path[i - 1], hamiltonian_path[i])

class TournamentGraph(DirectedGraph):
    def __init__(self):
        super().__init__()

    def is_tournament_graph(self) -> bool:
        return super().is_tournament_graph()

    def find_hamiltonian_path(self, scc: List[GraphNodeId]) -> List[GraphNodeId]:
      # must be a strongly connected component
      self.assert_is_strongly_connected_component(scc)

      # Complexity: O(len(scc)^2)
      hamiltonian_path: List[GraphNodeId] = [scc[0]]
      for k in range(1, len(scc)):
        # find first i < k | has_edge(hamiltonian_path[i], scc[k]) & has_edge(scc[k], hamiltonian_path[i+1])
        i = 0
        while i + 1 < k and not (self.has_edge(hamiltonian_path[i], scc[k]) and self.has_edge(scc[k], hamiltonian_path[i+1])):
          i += 1
        
        hamiltonian_path.insert(i + 1, scc[k])

      return hamiltonian_path

    def find_hamiltonian_cycle(self, hamiltonian_path_of_scc: List[GraphNodeId]) -> List[GraphNodeId]:
      self.assert_is_tournament_graph()
      self.assert_is_strongly_connected_component(hamiltonian_path_of_scc)
      self.assert_is_hamiltonian_path(hamiltonian_path_of_scc)

      # Complexity: O(len(hamiltonian_path)^2)
      accumulated_hamiltonian_cycle: List[GraphNodeId] = [hamiltonian_path_of_scc[0]]
      j = 1
      while j < len(hamiltonian_path_of_scc):
        p = j + 1
        r = -1
        found_backward_edge = False
        while not found_backward_edge and p < len(hamiltonian_path_of_scc):
          nonlocal r
          # check if there is a backward edge from hamiltonian_path[p] to accumulated_hamiltonian_cycle[r]
          r = 0 # the out-going node of the backward edge
          while r < len(accumulated_hamiltonian_cycle) and not self.has_edge(hamiltonian_path_of_scc[p], accumulated_hamiltonian_cycle[r]):
            r += 1
          
          if r < len(accumulated_hamiltonian_cycle):
            found_backward_edge = True
        
        # reorder the accumulated_hamiltonian_cycle: accumulated_hamiltonian_cycle[0 -> r - 1] -> hamiltonian_path_of_scc[j+1 -> p] -> accumulated_hamiltonian_cycle[r -> ...] -> hamiltonian_path_of_scc[0]
        j = p
        accumulated_hamiltonian_cycle = accumulated_hamiltonian_cycle[0:r] + hamiltonian_path_of_scc[j+1:p+1] + accumulated_hamiltonian_cycle[r:]

      return accumulated_hamiltonian_cycle