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
from schemas import OrderFairnessGraphNodeId, HashValue

class DirectedGraph:
    def __init__(self):
        self.nodes: Set[OrderFairnessGraphNodeId] = {}
        self.edges: Dict[OrderFairnessGraphNodeId, Set[OrderFairnessGraphNodeId]] = {}
        self.is_tournament_graph = None
        self.connected_components: List[Tuple[OrderFairnessGraphNodeId, HashValue]] = None

    def reset_graph_properties(self):
      self.is_tournament_graph = None
      self.connected_components = None

    def add_node(self, nodeId: OrderFairnessGraphNodeId):
        if nodeId in self.nodes:
            return
        self.nodes.add(nodeId)
        self.edges[nodeId] = set()
        # reset the graph properties
        self.reset_graph_properties()
    
    def add_directed_edge(self, nodeId1: OrderFairnessGraphNodeId, nodeId2: OrderFairnessGraphNodeId):
        if nodeId1 not in self.nodes:
            self.add_node(nodeId1)
        self.edges[nodeId1].add(nodeId2)
        # reset the graph properties
        self.reset_graph_properties()

    def has_edge(self, nodeId1: OrderFairnessGraphNodeId, nodeId2: OrderFairnessGraphNodeId) -> bool:
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
    
    def find_strongly_connected_components(self) -> List[List[OrderFairnessGraphNodeId]]:
        assert self.connected_components is None
        self.connected_components: List[Tuple[OrderFairnessGraphNodeId, HashValue]] = []
        
        # tarjan's algorithm
        index = 0
        indices = {}
        lowlinks = {}
        stack = []
        components: List[List[OrderFairnessGraphNodeId]] = []

        def strongconnect(nodeId: OrderFairnessGraphNodeId) -> None:
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

        # Store components with their hash values
        self.connected_components = components
        
        return components

    def assert_is_strongly_connected_component(self, hamiltonian_path: List[OrderFairnessGraphNodeId]):
      assert self.connected_components is not None
      hash_value = hash(tuple(sorted(hamiltonian_path)))
      assert hash_value in [connected_component[1] for connected_component in self.connected_components]

    def assert_is_hamiltonian_path(self, hamiltonian_path: List[OrderFairnessGraphNodeId]):
      for i in range(1, len(hamiltonian_path)):
        assert self.has_edge(hamiltonian_path[i - 1], hamiltonian_path[i])

class TournamentGraph(DirectedGraph):
    def __init__(self):
        super().__init__()

    def is_tournament_graph(self) -> bool:
        return super().is_tournament_graph()

    def find_hamiltonian_path(self, scc: List[OrderFairnessGraphNodeId]) -> List[OrderFairnessGraphNodeId]:
      # must be a strongly connected component
      self.assert_is_strongly_connected_component(scc)

      # Complexity: O(len(scc)^2)
      hamiltonian_path: List[OrderFairnessGraphNodeId] = [scc[0]]
      for k in range(1, len(scc)):
        # find first i < k | has_edge(hamiltonian_path[i], scc[k]) & has_edge(scc[k], hamiltonian_path[i+1])
        i = 0
        while i + 1 < k and not (self.has_edge(hamiltonian_path[i], scc[k]) and self.has_edge(scc[k], hamiltonian_path[i+1])):
          i += 1
        
        hamiltonian_path.insert(i + 1, scc[k])

      return hamiltonian_path

    def find_hamiltonian_cycle(self, hamiltonian_path_of_scc: List[OrderFairnessGraphNodeId]) -> List[OrderFairnessGraphNodeId]:
      self.assert_is_tournament_graph()
      self.assert_is_strongly_connected_component(hamiltonian_path_of_scc)
      self.assert_is_hamiltonian_path(hamiltonian_path_of_scc)

      # Complexity: O(len(hamiltonian_path)^2)
      accumulated_hamiltonian_cycle: List[OrderFairnessGraphNodeId] = [hamiltonian_path_of_scc[0]]
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