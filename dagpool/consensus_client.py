import asyncio
import random
from dataclasses import dataclass
from enum import Enum
from typing import Dict, Set, Optional, List, Tuple
import time
from queue import Queue
import random
import networkx as nx
import matplotlib.pyplot as plt
from matplotlib.colors import LinearSegmentedColormap
import hashlib
from schemas import TransactionId, PeerId, NodeId, NodeLabel
import json
from math import log2, ceil  # Added log2 and ceil imports

class Node:
    def __init__(self, peer_id: PeerId, round: int, is_witness: bool, newly_seen_txs_list: list[TransactionId], self_parent_hash: NodeId, cross_parent_hash: NodeId):
        self.peer_id = peer_id
        self.is_witness = is_witness
        self.round = round
        self.self_parent_hash = self_parent_hash
        self.cross_parent_hash = cross_parent_hash
        # TODO: migrate to Sparse Merkle Tree + proof of SMT transition
        self.newly_seen_txs_list = newly_seen_txs_list
        self.node_hash = self.hash_node(peer_id, round, is_witness, self_parent_hash, cross_parent_hash, newly_seen_txs_list)
        ## fork-related data
        self.equivocated_peers: Set[PeerId] = None # set of peers that current node believes are equivocated, and this node won't SEE (i.e. UNSEE) all nodes created by them. Note that this doesn't affect STRONGLY SEEING property of this node.
        self.seen_nodes: Set[NodeId] = None # set of nodes that current node sees
  
    def clone(self):
      return Node(self.peer_id, self.round, self.is_witness, [txs for txs in self.newly_seen_txs_list], self.self_parent_hash, self.cross_parent_hash)

    def label(self) -> NodeLabel:
      return f"{self.peer_id}:{self.node_hash}"
    
    @staticmethod
    def merkle_root_of_transaction_list(txs: list[TransactionId]) -> NodeId:
      assert len(txs) > 0
      # do the merkle tree construction using a while loop
      res = [tx for tx in txs]
      while len(res) > 1:
        new_res = []
        for i in range(0, len(res), 2):
          if i+1 < len(res):
            new_res.append(hashlib.sha256(f"{res[i]}{res[i+1]}".encode()).hexdigest())
          else:
            new_res.append(res[i])
        res = new_res
      return res[0]
    
    @staticmethod
    def hash_node(creator: PeerId, round: int, is_witness: bool, self_parent_hash: NodeId, cross_parent_hash: NodeId, newly_seen_txs_list: list[TransactionId]) -> NodeId:
        """Create deterministic hash for a node"""
        components = [creator, str(round), str(is_witness)]
        if cross_parent_hash:
            components.append(cross_parent_hash)
        if self_parent_hash:
            components.append(self_parent_hash)
        if newly_seen_txs_list:
            components.append(Node.merkle_root_of_transaction_list(newly_seen_txs_list))
        return hashlib.sha256(''.join(components).encode()).hexdigest()[:8] # the hash value of a node basically depends deterministically on all of its content

    def verify_node_hash(self) -> bool:
      return self.node_hash == Node.hash_node(self.peer_id, self.round, self.is_witness, self.self_parent_hash, self.cross_parent_hash, self.newly_seen_txs_list)

    def is_genesis(self):
      is_genesis = self.round == 0 and self.is_witness == True and self.self_parent_hash == "" and self.cross_parent_hash == "" and self.verify_node_hash()
      return is_genesis

    def is_non_genesis(self):
      return not self.is_genesis()

    def __str__(self):
      return f"{"GENESIS " if self.is_genesis() else ""}Node(node_hash={self.node_hash}, peer_id={self.peer_id}, round={self.round}, is_witness={self.is_witness}, self_parent_hash={self.self_parent_hash}, cross_parent_hash={self.cross_parent_hash}, newly_seen_txs_list={self.newly_seen_txs_list})" # , equivocated_peers={self.equivocated_peers}, seen_nodes={self.seen_nodes})"

class ConnectionState(Enum):
    CLOSED = 0
    SYN_SENT = 1
    SYN_RECEIVED = 2
    ESTABLISHED = 3
    FIN_WAIT = 4

@dataclass
class Checkpoint:
    timestamp: float
    nodes: Dict[PeerId, Node]  # peer_id -> Node mapping
    accumulated_txs: Set[TransactionId]  # All transactions up to this checkpoint

    def verify_dag_structure(self) -> bool:
        """Verify that the checkpoint forms a valid DAG structure"""
        for node in self.nodes.values():
            if node.self_parent_hash and node.self_parent_hash.peer_id != node.peer_id:
                return False
            if node.self_parent_hash and node.self_parent_hash.round >= node.round:
                return False
            if node.cross_parent_hash and node.cross_parent_hash.round >= node.round:
                return False
        return True

class NetworkSimulator:
    def __init__(self, latency_ms_range=(50, 200), packet_loss_prob=0.1, random_instance: random.Random=random.Random(0)):
        self.connections: Dict[tuple, ConnectionState] = {}
        self.latency_range = latency_ms_range
        self.packet_loss_prob = packet_loss_prob
        self.random_instance = random_instance
        # this NetworkSimulator also works like a "beacon chain" to manage the common knowledge of all peers
        self.peers: List['ConsensusPeer'] = []
        # Checkpoint-related attributes
        self.checkpoints: List[Checkpoint] = []
        self.genesis_checkpoint: Optional[Checkpoint] = None
        self.last_checkpoint_round = 0

        # global mempool: simulate a global mempool of all transactions from all clients
        self.global_mempool = set()

    def register_peer(self, peer: 'ConsensusPeer'):
        # peer_id must be unique
        assert peer.peer_id not in [p.peer_id for p in self.peers]
        self.peers.append(peer)

    def get_all_peer_ids(self) -> List[PeerId]:
        return [p.peer_id for p in self.peers]

    def unregister_peer(self, peer: 'ConsensusPeer'):
        assert peer in self.peers
        self.peers.remove(peer)

    async def register_genesis_nodes(self):
        """Register genesis nodes from all peers as the first checkpoint"""
        # Create genesis nodes and initial gossip
        genesis_nodes: Dict[PeerId, Node] = {}
        for peer1 in self.peers:
            genesis_node = peer1.create_genesis_node()

            genesis_nodes[peer1.peer_id] = genesis_node
            for peer2 in self.peers:
              if peer2.peer_id == peer1.peer_id:
                continue

              cloned_genesis_node = genesis_node.clone() # simulate the process of serializing and deserializing the nodes in internet protocols

              # Gossip genesis node to neighbors
              success = await self.gossip_send_node_and_ancestry(peer1.peer_id, peer2.peer_id, cloned_genesis_node)
              if not success:
                print(f"peer {peer1.peer_id} gossiped send to {peer2.peer_id} genesis node {genesis_node.node_hash} failed")
              else:
                print(f"peer {peer1.peer_id} gossiped send to {peer2.peer_id} genesis node {genesis_node.node_hash} successfully")

        self.genesis_checkpoint = Checkpoint(
            timestamp=time.time(),
            nodes=genesis_nodes.copy(),
            accumulated_txs=set()  # Empty at genesis
        )
        self.checkpoints.append(self.genesis_checkpoint)

    # TODO: implement checkpoint mechanism

    def new_txs_from_user_client(self) -> (list[TransactionId], list['ConsensusPeer']):
        """Pick a random transaction from the global mempool that will be sent to random peers"""
        num_txs = self.random_instance.randint(1, 10)

        mempool_txs = sorted(self.global_mempool)
        txs = []
        for _ in range(num_txs):
          # 50% pick a random txs already in the global mempool
          if len(mempool_txs) > 0 and self.random_instance.random() < 0.5:
            txs.append(self.random_instance.choice(mempool_txs))
          else:
            # 50% add a new txs
            new_txs = f"tx_{len(self.global_mempool) + 1}"
            txs.append(new_txs)
            self.global_mempool.add(new_txs)
        
        # pick random peers from the network to send the txs to
        # choose ⌈log2(log2(N))⌉ peers on average (because usually a client only sends to 1 rpc portal)
        num_peers = len(self.peers)
        num_peers_to_send = max(1, ceil(log2(log2(num_peers))))
        peer_ids_to_send = self.random_instance.sample([p.peer_id for p in self.peers], num_peers_to_send)

        return txs, [peer for peer in self.peers if peer.peer_id in peer_ids_to_send]

    def get_accumulated_txs_until_node(self, node: Node) -> Set[TransactionId]:
        """Get all transactions accumulated up to a specific node"""
        # First, find the latest checkpoint before this node
        latest_applicable_checkpoint = None
        for checkpoint in reversed(self.checkpoints):
            if checkpoint.round < node.round:
                latest_applicable_checkpoint = checkpoint
                break

        accumulated_txs = set()
        if latest_applicable_checkpoint:
            accumulated_txs.update(latest_applicable_checkpoint.accumulated_txs)

        # Add transactions from the node and its ancestors back to the checkpoint
        def collect_txs(current_node):
            if not current_node or (latest_applicable_checkpoint and 
                                  current_node.round <= latest_applicable_checkpoint.round):
                return
            accumulated_txs.update(current_node.newly_seen_txs_list)
            if current_node.self_parent_hash:
                collect_txs(current_node.self_parent_hash)
            if current_node.cross_parent_hash:
                collect_txs(current_node.cross_parent_hash)

        collect_txs(node)
        return accumulated_txs

    async def connect(self, peer_a: PeerId, peer_b: PeerId):
        """Simulate TCP three-way handshake"""
        conn_key = (peer_a, peer_b)

        # SYN
        if self.random_instance.random() > self.packet_loss_prob:
            self.connections[conn_key] = ConnectionState.SYN_SENT
            await self._delay()
            
            # SYN-ACK
            if self.random_instance.random() > self.packet_loss_prob:
                self.connections[conn_key] = ConnectionState.SYN_RECEIVED
                await self._delay()
                # ACK
                if self.random_instance.random() > self.packet_loss_prob:
                    self.connections[conn_key] = ConnectionState.ESTABLISHED

                    ## update neighbors
                    ## TODO: move this to a better place
                    peer_a_peer: ConsensusPeer = [p for p in self.peers if p.peer_id == peer_a][0]
                    peer_b_peer: ConsensusPeer = [p for p in self.peers if p.peer_id == peer_b][0]

                    if peer_a_peer.peer_id not in peer_b_peer.neighbors:
                      peer_b_peer.neighbors.append(peer_a_peer.peer_id)
                    if peer_b_peer.peer_id not in peer_a_peer.neighbors:
                      peer_a_peer.neighbors.append(peer_b_peer.peer_id)

                    return True

        self.connections[conn_key] = ConnectionState.CLOSED
        return False

    def is_connected(self, peer_a: PeerId, peer_b: PeerId) -> bool:
        conn_key = (peer_a, peer_b)
        conn_key_rev = (peer_b, peer_a)
        return self.connections.get(conn_key, ConnectionState.CLOSED) == ConnectionState.ESTABLISHED or self.connections.get(conn_key_rev, ConnectionState.CLOSED) == ConnectionState.ESTABLISHED

    async def disconnect(self, peer_a: PeerId, peer_b: PeerId):
        """Simulate TCP connection termination"""
        conn_key = (peer_a, peer_b)
        if conn_key in self.connections:
            self.connections[conn_key] = ConnectionState.FIN_WAIT
            await self._delay()
            self.connections.pop(conn_key)

    async def gossip_send_node_and_ancestry(self, sender: PeerId, receiver: PeerId, node1: Node) -> bool:
        """Gossip a node and its ancestry to a peer"""
        assert sender != receiver

        for _ in range(3):
          if not self.is_connected(sender, receiver):
            if _ < 2:
              await self.connect(sender, receiver)
            else:
              print(f"peer {sender} can't connect to {receiver} even after 2 attempts")
              return False # can't help any more

        ### TODO: migrate this to efficient proof-based gossip where sender does not have to send the whole ancestry of node1 backwards to node2

        sender_peer = [p for p in self.peers if p.peer_id == sender][0]
        receiver_peer = [p for p in self.peers if p.peer_id == receiver][0]
  
        receiver_has_all_needed_ancestors = False
        all_received_nodes = []
        current_gossiped_list = [node1] # we don't send the ancestry of the last node of the sender because node1 might be an equivocated node
        while not receiver_has_all_needed_ancestors:
          # receiver receives nodes from sender
          receiver_has_all_needed_ancestors = True

          new_gossiped_list = []
          for node in current_gossiped_list:
            if receiver_peer.has_seen_valid_node(node):
              continue
            else:
              all_received_nodes.append(node)
              receiver_has_all_needed_ancestors = False # not stop

              if not node.is_genesis():
                self_parent_node = sender_peer.get_node_by_hash(node.self_parent_hash)
                cross_parent_node = sender_peer.get_node_by_hash(node.cross_parent_hash)
                assert self_parent_node is not None and cross_parent_node is not None

                # use clone() to simulate the process of serializing and deserializing the nodes in internet protocols
                new_gossiped_list.append(self_parent_node.clone())
                new_gossiped_list.append(cross_parent_node.clone())
          
          current_gossiped_list = new_gossiped_list

        ### reverse all_received_nodes because the early nodes in the ancestry are in the right of the list, but we want them to be in the left
        all_received_nodes = all_received_nodes[::-1]
    
        for i in range(len(all_received_nodes)):
          current_node = all_received_nodes[i]
          if not receiver_peer.verify_node_and_add_to_local_view(current_node):
            continue

        return receiver_peer.has_seen_valid_node(node1)

    async def _delay(self):
        """Simulate network latency"""
        delay = 0
        # TODO: turn on delay (a value in the self.latency_range range) for full simulation
        await asyncio.sleep(delay)

    def find_node_by_hash(self, node_hash: str) -> Optional[Node]:
        """Find a node by its hash across all peers"""
        if not node_hash:
            return None
        for peer in self.peers:
            for node in peer.my_nodes():
                if node.node_hash == node_hash:
                    return node
        return None

class ConsensusPeer:
    def __init__(self, peer_id: PeerId, is_adversary: bool, seed: int, network: NetworkSimulator):
        self.peer_id = peer_id
        self.is_adversary = is_adversary
        self.random_instance = random.Random(seed)
        self.current_round = 0
        self.seen_valid_nodes: Dict[PeerId, List[Node]] = {}
        self.accumulated_txs = set()  # Track all transactions seen by this peer
        self.network = network
        ## adversary-related data
        self.equivocated_peers: Set[PeerId] = set() # set of peers that current peer believes they actively create equivocated nodes
        self.equivocated_nodes: Set[NodeId] = set() # set of nodes that current peer believes are equivocated
        self.equivocation_prob = 0.5 if is_adversary else 0.0
        self.neighbors: list[PeerId] = []  # Track neighboring peers
        # if each peer connects to log(N) neighbors, a transaction would takes O(log(N)/log(log(N))) gossip hops to reach the whole network
        # for N = 10^6, it would be 7 hops
        self.tx_receive_times = {}  # Track when transactions were received
        self.pending_txs: List[Tuple[TransactionId, float]] = []
        self.local_graph: Dict[NodeId, Set[NodeId]] = {} # node_hash to set of node_hashes (parent, child)
    
    def my_nodes(self) -> List[Node]:
      return self.seen_valid_nodes[self.peer_id]
    
    def get_my_last_node(self):
      return self.my_nodes()[-1]

    def get_seen_valid_nodes(self) -> List[Node]:
      res = []
      for peer_id in self.seen_valid_nodes:
        res.extend(self.seen_valid_nodes[peer_id])
      return res
    
    def count_seen_valid_nodes(self):
      return len(self.get_seen_valid_nodes())

    def get_predecessors(self, node: Node) -> list[Node]:
      if node.is_genesis():
        return []
      
      predecessors = []
      self_parent_node = self.get_node_by_hash(node.self_parent_hash)
      cross_parent_node = self.get_node_by_hash(node.cross_parent_hash)
      assert self_parent_node is not None and cross_parent_node is not None
      predecessors.append(self_parent_node)
      predecessors.append(cross_parent_node)
      return predecessors
    
    def get_successors(self, node: Node) -> list[Node]:
      successors = []
      for successor in self.local_graph.get(node.node_hash, set()):
        successors.append(self.get_node_by_hash(successor))
      return sorted(successors, key=lambda x: x.round)

    def get_ancestry(self, node: Node) -> Set[Node]:
      ancestry: Dict[NodeId, Node] = {}
      def dfs_backwards(node: Node):
        if node.node_hash in ancestry:
          return
        ancestry[node.node_hash] = node
        for predecessor in self.get_predecessors(node):
          dfs_backwards(predecessor)
      dfs_backwards(node)
      return set(ancestry.values())

    def get_lineage(self, node: Node) -> Set[Node]:
      lineage: Dict[NodeId, Node] = {}
      def dfs_forwards(node: Node):
        if node.node_hash in lineage:
          return
        lineage[node.node_hash] = node
        for successor in self.get_successors(node):
          dfs_forwards(successor)
      dfs_forwards(node)
      return set(lineage.values())
    
    def get_graph_info(self) -> Dict[NodeId, Node]:
      res: Dict[NodeId, Node] = {}
      for peer_id in self.seen_valid_nodes:
        for node in self.seen_valid_nodes[peer_id]:
          res[node.node_hash] = node

      return res

    def visualize_view(self):
      print(f"peer {self.peer_id} sees the DAG:")
      for peer_id in self.seen_valid_nodes:
        for node in self.seen_valid_nodes[peer_id]:
          print(f"{node}")

      # pos = {}
      round_colors = {
          0: '#8b0000',  # Dark red for round 0
          1: '#ff6600',  # Orange for round 1
          2: '#b7950b',  # Yellow for round 2
          3: '#00cc00',  # Green for round 3
          4: '#0066cc',  # Blue for round 4
          5: '#6600cc',  # Purple for round 5
          6: '#008080',  # Teal for round 6
          7: '#CD5C5C',  # IndianRed for round 7
          8: '#DE3163',  # #DE3163 for round 8
          9: '#800080',  # #800080 for round 9
      }

      # Assign positions to nodes
      peers = self.network.peers
      node_positions: Dict[NodeId, Tuple[int, int]] = {}
      seen_valid_nodes = self.get_seen_valid_nodes()
      
      count_lines = 0
      lines_of_peers: Dict[PeerId, list[int]] = {}
      count_self_direct_children = {} # including the equivocated self direct children

      # initial line id for each peer and its nodes
      for node in seen_valid_nodes:
        peer_id = node.peer_id
        line_id = -1
        if peer_id not in lines_of_peers:
          count_lines += 1
          lines_of_peers[peer_id] = [count_lines]
          line_id = count_lines
        else:
          line_id = lines_of_peers[peer_id][0]

        node_positions[node.node_hash] = (0, line_id)

      # Construct the DiGraph for plotting
      edges = []
      G = nx.DiGraph()
      
      visited = {}
      # Adjust the positioning to ensure chronological order
      def adjust_position(node, visited):
        nonlocal count_lines
        if node.node_hash in visited:
          return
        visited[node.node_hash] = True

        for predecessor in self.get_predecessors(node):
          edges.append((predecessor.node_hash, node.node_hash))
          adjust_position(predecessor, visited)
          line_id = node_positions[node.node_hash][1] # keep current line id

          ## calculate line id for the current node if the predecessor is self-parent and it has equivocated children
          if predecessor.node_hash == node.self_parent_hash:
            if predecessor.node_hash not in count_self_direct_children:
              count_self_direct_children[predecessor.node_hash] = 0
            count_self_direct_children[predecessor.node_hash] += 1
            if count_self_direct_children[predecessor.node_hash] > 1: # self parent has equivocated children
              count_lines += 1
              lines_of_peers[predecessor.peer_id].append(count_lines)
              line_id = count_lines
          
          node_positions[node.node_hash] = (max(node_positions[predecessor.node_hash][0] + 3, node_positions[node.node_hash][0]), line_id)

      for node in seen_valid_nodes:
        if not node.node_hash in visited:
          adjust_position(node, visited)

      G.add_edges_from(edges)

      ## Rescale the y-coordinates based on the line ids from all nodes
      line_id_to_peer: Dict[int, PeerId] = {}
      for peer_id in lines_of_peers:
        for line_id in lines_of_peers[peer_id]:
          line_id_to_peer[line_id] = peer_id

      all_line_ids = sorted(set([node_positions[node.node_hash][1] for node in seen_valid_nodes]), key=lambda x: (line_id_to_peer[x], x))
      remapped_line_ids: Dict[int, int] = {}
      for i in range(len(all_line_ids)):
        remapped_line_ids[all_line_ids[i]] = i

      for node in seen_valid_nodes:
        node_positions[node.node_hash] = (node_positions[node.node_hash][0], - 2 * remapped_line_ids[node_positions[node.node_hash][1]])

      # Draw nodes and edges
      plt.figure(figsize=(50, 10))  # Increase figure size for better visibility

      for node in seen_valid_nodes:
          round_number = node.round
          node_color = round_colors.get(round_number, 'gray')
          nx.draw_networkx_nodes(
              G,
              node_positions,
              nodelist=[node.node_hash],
              node_size=1000,
              node_shape='s',
              node_color=node_color,
              label=[node.peer_id]
          )

      nx.draw_networkx_edges(G, node_positions, edge_color='black', arrows=True, arrowsize=20, width=1, node_size=1000)  # Ensure arrows are properly sized relative to nodes

      # Draw node labels with a border
      for node in seen_valid_nodes:
        x, y = node_positions[node.node_hash]
        # Draw the text multiple times with slight offsets to create a border
        # for dx, dy in [(-0.5, -0.5), (-0.5, 0.5), (0.5, -0.5), (0.5, 0.5)]:
        #     plt.text(x + dx * 0.01, y + dy * 0.01, node.node_hash, fontsize=10, ha='center', va='center', color='black')
        # Draw the actual text in white
        plt.text(x, y, node.label(), fontsize=5, ha='center', va='center', color='white')


      # Draw horizontal separator lines between different peers
      unique_lines = sorted(remapped_line_ids.values())
      peer_boundaries = set()

      for i in range(len(unique_lines) - 1):
          peer1 = line_id_to_peer[all_line_ids[i]]
          peer2 = line_id_to_peer[all_line_ids[i + 1]]
          if peer1 != peer2:  # If the next line belongs to a different peer, draw a separator
              boundary_y = -2 * unique_lines[i] - 1  # Slightly below the last line of peer1
              peer_boundaries.add(boundary_y)

      for y in peer_boundaries:
          plt.plot([min(x for x, _ in node_positions.values()), 
                    max(x for x, _ in node_positions.values())], 
                  [y, y], color='black', linestyle='dashed', linewidth=1)
      
      # plot the figure
      plt.axis('off')
      plt.show()

    def get_max_num_neighbors(self):
      return ceil(log2(len(self.network.peers)))

    def create_genesis_node(self):
        """Create a genesis/bootstrap node"""
        node = Node(
            peer_id=self.peer_id,
            round=0,
            is_witness=True,
            newly_seen_txs_list=[],
            self_parent_hash="",
            cross_parent_hash=""
        )
        assert self.verify_node_and_add_to_local_view(node) == True
        return node

    # TODO: implement bootstrap node (first node refers to parents in a checkpoint after a node rejoins the network)
    
    def has_seen_valid_node(self, node: Node) -> bool:
      if node.peer_id == self.peer_id:
        return node in self.my_nodes()
      else:
        return (node.peer_id in self.seen_valid_nodes) and (node.node_hash in [node.node_hash for node in self.seen_valid_nodes[node.peer_id]])

    def get_node_by_hash(self, node_hash: NodeId) -> Optional[Node]:
      for node in self.my_nodes():
        if node.node_hash == node_hash:
          return node
      ## find in seen_valid_nodes of other peers
      for peer_id in self.seen_valid_nodes:
        for node in self.seen_valid_nodes[peer_id]:
          if node.node_hash == node_hash:
            return node
      return None

    def record_transaction_receipt(self, tx_id: TransactionId, timestamp: float):
        """Record when a transaction was received"""
        self.tx_receive_times[tx_id] = timestamp
        self.pending_txs.append((tx_id, timestamp))

    def select_neighbors(self, all_peers: List[PeerId]):
        """Randomly select neighbors from available peers"""
        potential_neighbors = [p for p in all_peers if p != self.peer_id]
        num_neighbors = min(self.get_max_num_neighbors(), len(all_peers) - 1)
        self.neighbors = self.random_instance.sample(potential_neighbors, num_neighbors)

    def compute_seen_nodes_of_new_node(self, node: Node):
      """Compute the list of seen_nodes of the new node"""
      assert node.seen_nodes is None and node.equivocated_peers is None
      node.seen_nodes = set()
      equivocated_peers = set()

      ancestry_of_node = self.get_ancestry(node)
      self_parent_set: Set[NodeId] = set()

      for cur_node in ancestry_of_node:
        if not cur_node.is_genesis():
          if cur_node.self_parent_hash in self_parent_set:
            equivocated_peers.add(cur_node.peer_id)
          else:
            self_parent_set.add(cur_node.self_parent_hash)

      for cur_node in ancestry_of_node:
        if cur_node.peer_id in equivocated_peers:
          continue
        
        node.seen_nodes.add(cur_node.node_hash)
      node.equivocated_peers = equivocated_peers

      return

    def verify_node_and_add_to_local_view(self, node: Node = None) -> bool:
        """Verify a node and its transactions, and add it to the local view"""
        if not self.verify_node(node):
          return False

        # self.add_node_to_local_view(node)
        
        # add to seen_valid_nodes
        if node.peer_id not in self.seen_valid_nodes:
          self.seen_valid_nodes[node.peer_id] = []

        should_add_node = node.node_hash not in [node.node_hash for node in self.seen_valid_nodes[node.peer_id]]
        if should_add_node:
          self.seen_valid_nodes[node.peer_id].append(node)

          for predecessor in self.get_predecessors(node):
            if predecessor.node_hash not in self.local_graph:
              self.local_graph[predecessor.node_hash] = set()
            self.local_graph[predecessor.node_hash].add(node.node_hash)

          # compute the list of seen_nodes of the new node
          self.compute_seen_nodes_of_new_node(node)
          # do the cleanup if the node is created by the current peer
          if node.peer_id == self.peer_id:
            self.pending_txs.clear() # because all txs in the pending_txs are now in the new node
            self.current_round = node.round # this makes the current round of the peer = the round of the last node in the list of its nodes
    
        print(f"Peer {self.peer_id} added node {node.node_hash} to its local view => new round = {self.current_round}")

        return True

    def get_strongly_seen_valid_witnesses(self, dest_node: Node, r: int) -> list["Node"]:
        ## check if this witness strongly sees > 2/3 of witnesses of r
        ## if some witnesses are descendants of equivocated nodes, they are ignored completely
        ## NOTE: we already make sure the ancestry of dest_node is verified
        ## NOTE: at this point, dest_node is not called compute_seen_nodes_of_new_node() yet

        strongly_sees_threshold = 2/3 * len(self.network.peers)
        strongly_seen_witnesses: list["Node"] = []
        ancestry_of_dest_node = self.get_ancestry(dest_node)
        # sorted deterministically
        witnesses_in_round_r = sorted([node for node in ancestry_of_dest_node if node.is_witness and node.round == r], key=lambda x: (x.peer_id, x.node_hash))

        # itearate through all witnesses in round r and check if the dest_node can strongly see them
        for witness in witnesses_in_round_r:
          lineage_of_witness = self.get_lineage(witness)
          crossed_peers = set()
          can_conclude_strongly_seen = False
          for mid_node in lineage_of_witness:
            if (mid_node.peer_id in crossed_peers) or (mid_node not in ancestry_of_dest_node):
              continue # can include the witness itself if satisfies the condition
            
            should_count_as_valid_path_from_witness_to_dest_node = (mid_node.node_hash == dest_node.node_hash) or (witness.node_hash in mid_node.seen_nodes)
            if should_count_as_valid_path_from_witness_to_dest_node:
              crossed_peers.add(mid_node.peer_id)
              can_conclude_strongly_seen = len(crossed_peers) > strongly_sees_threshold
            
            if can_conclude_strongly_seen:
              break

          if can_conclude_strongly_seen:
            if witness.peer_id not in [node.peer_id for node in strongly_seen_witnesses]:
              # at most 1 witness per peer is counted
              strongly_seen_witnesses.append(witness)

        return strongly_seen_witnesses
    
    def check_round_number_of_non_genesis_node(self, node: Node) -> bool:
      """
      if a node is of round r:
        - it must not strongly sees > 2N/3 of witnesses of round r
        - if its self parent is of round r, it is valid. if its self parent is of round r-1, it must strongly sees > 2N/3 of witnesses of round r-1
      """
      N = len(self.network.peers)
      r = node.round

      # the node must not strongly sees > 2/3 of witnesses of round r
      strongly_seen_witnesses_in_round_r = self.get_strongly_seen_valid_witnesses(node, r)

      if len(strongly_seen_witnesses_in_round_r) > 2 * N / 3:
        return False

      # check non-witness node case
      self_parent_node = self.get_node_by_hash(node.self_parent_hash)
      assert self_parent_node is not None
      if self_parent_node.round == r:
        return True

      # check witness node case
      strongly_seen_witnesses_in_round_r_minus_1 = self.get_strongly_seen_valid_witnesses(node, r-1)

      return len(strongly_seen_witnesses_in_round_r_minus_1) > 2 * N / 3

    def verify_node(self, node: Node = None) -> bool:
        """Verify a node and its transactions
        - round number must be valid
        - node hash must be valid
        => This method should be called recursively for all ancestors of a node before it's verified
        """
        if node is None:
          return False

        if node.is_genesis():
          return True

        if not node.verify_node_hash():
          return False

        return self.check_round_number_of_non_genesis_node(node)

    def get_all_transactions(self) -> Set[TransactionId]:
        """Get all transactions known to this peer"""
        return self.accumulated_txs.copy()

    def get_all_seen_txs_up_to_a_verified_node(self, node: Node) -> Set[TransactionId]:
      """Get all transactions seen through this node"""
      res = set()
      while node is not None:
        res.update(node.newly_seen_txs_list)
        node = self.get_node_by_hash(node.self_parent_hash)
      return res

    def calculate_newly_seen_txs_list_of_new_node(self, self_parent: Node, cross_parent: Node, pending_txs: List[Tuple[TransactionId, float]]) -> list[TransactionId]:
        """Calculate the newly seen transactions for a new node"""
        all_seen_txs_up_to_self_parent = self.get_all_seen_txs_up_to_a_verified_node(self_parent)
        all_seen_txs_up_to_cross_parent = self.get_all_seen_txs_up_to_a_verified_node(cross_parent)

        return sorted((set([txs for txs, _ in pending_txs]) | all_seen_txs_up_to_cross_parent) - all_seen_txs_up_to_self_parent)

    def compute_new_node(self, self_parent: Node, cross_parent: Node) -> Node:
        """A view-only method that computes a new node based on the self parent and cross parent
        @return: the newly created node, if there is no new txs, return None
        """
        assert self_parent is not None and cross_parent is not None
        assert self_parent == self.get_my_last_node()

        # the newly seen list of txs in the new node must be not empty
        # TODO: sort this list by timestamp of receipt of the transactions
        newly_seen_txs_list: List[TransactionId] = list(self.calculate_newly_seen_txs_list_of_new_node(self_parent, cross_parent, self.pending_txs))

        if len(newly_seen_txs_list) <= 0:
          # can't extend the node sequence because there is no new txs, this is to save the network capacity
          print(f"Peer {self.peer_id} can't extend the node sequence because there is no new txs, this is to save the network capacity, self_parent = {self_parent.node_hash}, cross_parent = {cross_parent.node_hash}")
          return None

        round_num = len(self.my_nodes())
        base_hash = f"{self.peer_id}{str(round_num).zfill(3)}"

        assert self_parent.round == self.current_round

        new_node = Node(
            peer_id=self.peer_id,
            round=self_parent.round,
            is_witness=False,
            newly_seen_txs_list=newly_seen_txs_list,
            self_parent_hash=self_parent.node_hash,
            cross_parent_hash=cross_parent.node_hash
        )

        if not self.verify_node(new_node):
          new_node = Node(
            peer_id=self.peer_id,
            round=self_parent.round + 1,
            is_witness=True,
            newly_seen_txs_list=newly_seen_txs_list,
            self_parent_hash=self_parent.node_hash,
            cross_parent_hash=cross_parent.node_hash
          )
          assert self.verify_node(new_node)

        print(f"Peer {self.peer_id} COMPUTED NEW NODE {new_node.node_hash} from {self_parent.node_hash} and {cross_parent.node_hash}")

        return new_node

    async def gossip_push(self):
        """Sync with another peer through gossip, potentially sending different views"""
        # generate a random permutation of connected peers
        # try to extend the node sequence and push it to the neighbors

        self_parent_node = self.get_my_last_node()
        # pick a random peer with non-empty seen_valid_nodes
        possible_cross_peers = [peer_id for peer_id in self.seen_valid_nodes if self.seen_valid_nodes[peer_id]]

        if len(possible_cross_peers) <= 1:
          return # can't extend the node sequence because there is no cross parent for the new node

        randomness = min([self.network.random_instance.random(), self.network.random_instance.random()])
        num_nodes_to_create = min(len(possible_cross_peers), 1 + (1 if randomness < self.equivocation_prob else 0))
        new_nodes = [] # if there are more than 1 node in this list, they are equivocated nodes and that means current peer is an adversary

        # NOTE: currently, the equivocation logic is simple, an adversary basically picks the last node of the current peer as the self parent, and the latest nodes of different cross peers as the cross parents
        
        for _ in range(num_nodes_to_create):
          found_unique_cross_parent = False
          while not found_unique_cross_parent:
            found_unique_cross_parent = True

            cross_parent_peer_id = self.random_instance.choice(possible_cross_peers)
            while cross_parent_peer_id == self.peer_id:
              cross_parent_peer_id = self.random_instance.choice(possible_cross_peers)
            
            cross_parent_node = self.seen_valid_nodes[cross_parent_peer_id][-1]

            new_node = self.compute_new_node(self_parent=self_parent_node, cross_parent=cross_parent_node)
            if new_node is None:
              return

            if new_node.cross_parent_hash not in [node.cross_parent_hash for node in new_nodes]:
              new_nodes.append(new_node)
            else:
              found_unique_cross_parent = False
        
        assert len(new_nodes) == num_nodes_to_create
        for new_node in new_nodes:
          assert self.verify_node_and_add_to_local_view(new_node)

        # start gossiping to neighboring peers
        for i in range(len(self.neighbors)):
            other_peer_id = self.neighbors[i]
            # select randomly nodes from new_nodes
            node_to_send = (new_nodes[0] if i * 2 < len(self.neighbors) else new_nodes[-1]).clone() # simulate the process of serializing and deserializing the nodes in internet protocols

            # Send the selected node
            success = await self.network.gossip_send_node_and_ancestry(self.peer_id, other_peer_id, node_to_send)
            if not success:
              print(f"peer {self.peer_id} gossiped send to {other_peer_id} node {node_to_send.node_hash} failed")
            else:
              print(f"peer {self.peer_id} gossiped send to {other_peer_id} node {node_to_send.node_hash} successfully")
            
    # TODO: finish equivocation detection
    def detect_equivocation(self) -> list:
      pass

async def main():
    # Create network simulator
    network = NetworkSimulator(
        latency_ms_range=(50, 200),
        packet_loss_prob=0.1,
        random_instance=random.Random(0)
    )

    # Create peers
    num_peers = 4 # next threshold for count_adversary = 2 is N = 7
    count_adversary = 0
    for i in range(num_peers):
        is_adversary = (count_adversary + 1) < 1 * num_peers / 3 and network.random_instance.random() < 0.5

        if is_adversary:
          count_adversary += 1
        
        peer = ConsensusPeer(
            peer_id=f"P{i}",
            is_adversary=is_adversary,
            seed=i,
            network=network
        )
        network.register_peer(peer)
    peers = network.peers

    # Initialize peer neighborhoods
    all_peer_ids = network.get_all_peer_ids()
    for peer in peers:
        peer.select_neighbors(all_peer_ids)

    # Register genesis checkpoint
    await network.register_genesis_nodes()

    MIN_NUM_ROUNDS = 10
    current_simluated_timestamp = 0

    # Main consensus loop
    i = 0
    while True and i < 100:
        i += 1
        if i % 100 == 0:
          print(f"{i}th iteration")
        # Count peers that have reached MIN_NUM_ROUNDS rounds
        peers_completed = sum(1 for c in peers if c.current_round >= MIN_NUM_ROUNDS)
        if peers_completed > (2 * num_peers // 3):
            break

        # Randomly select an action for a random peer
        action = network.random_instance.random()

        if action < 0.15:  # Generate new transactions
            txs, peers_to_send = network.new_txs_from_user_client()
            for tx in txs:
              for peer in peers_to_send:
                if network.random_instance.random() < 0.3:
                  current_simluated_timestamp += 1 # advance current timestamp
                
                # Record receipt time for self
                peer.record_transaction_receipt(tx, current_simluated_timestamp)
        else:
          # pick a random set of peers to do gossip push from it to its neighbors
          random_peers = network.random_instance.sample(peers, network.random_instance.randint(1, len(peers)))
          for peer in random_peers:
              await peer.gossip_push()

        # Create checkpoints periodically
        # TODO: create network checkpoints dynamically via network.create_checkpoint()

    for peer in peers:
      peer.visualize_view()
      if peer.is_adversary:
        print(f"Peer {peer.peer_id} is an adversary")
      print(f"Neighbors of {peer.peer_id}: {peer.neighbors}")
    print(f"Consensus completed with first {peers_completed} peers reaching round {MIN_NUM_ROUNDS}")

    def validate_consistency():
      graph_info_list = [peer.get_graph_info() for peer in peers]

      found_node_conflict = False
      global_info_of_node: Dict[NodeId, str] = {}

      for i in range(len(graph_info_list)):
        for node_id in graph_info_list[i]:
          node_description = graph_info_list[i][node_id].__str__()
          if node_id in global_info_of_node:
            if global_info_of_node[node_id] != node_description:
              found_node_conflict = True
              print(f"FAILED: found conflict in node info of node {node_id}: {node_description} vs {global_info_of_node[node_id]}")
          else:
            global_info_of_node[node_id] = node_description

          assert found_node_conflict == False
      
      print("SUCCESS: There is no conflict in node info between peers")

    validate_consistency()

# Run the simulation
asyncio.run(main())

### Possible attacks:
# Long-Range Attacks: If validators controlling past checkpoints sell their keys, an attacker can re-sign an alternative history, leading to checkpoint reversals.
# => Dangerous once attacker can control > 2/3 of the OLD validators
# Majority Takeover: If an attacker gains control of 2/3 of the validators (BFT threshold), they could re-finalize a new chain with different checkpoints.
# => recursive validity proof + proof of finality
# Solution: Post-Unstaking Slashing for X blocks after unstaking (but not able to withdraw before X blocks yet)

# [] TODO: finish gossip-DAG architecture
# [] TODO: finish DAGPool's order fairness gadget