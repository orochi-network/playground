import asyncio
import sys
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
from schemas import TransactionId, PeerId, NodeId, NodeLabel, Signature, BatchId, Pubkey, HashValue
import json
from math import log2, ceil  # Added log2 and ceil imports
from graph import SemiCompleteDiGraph

import builtins
original_print = builtins.print

SHOULD_PRINT = True
def print(msg, force=False):
  if SHOULD_PRINT or force:
    original_print(msg)

printed = set()

with open("debug.txt", "w") as f:
  f.write("")

def ddebug(peer_id: PeerId, *args, **kwargs):
  # if SHOULD_PRINT:
    # write to file debug.txt
    with open("debug.txt", "a") as f:
      line = f"{peer_id}: {' '.join([str(arg) for arg in args])}\n"
      if line not in printed:
        f.write(line)
        # printed.add(line)

BEACON_PACE = 10
# the first BEACON_PACE rounds are derived directly from the list of peers
# BEACON_PACE should be chosen large enough to make sure peers have enough time to realize that they are the leader of the next rounds
BEACON_FIELD_PRIME = 28948022309329048855892746252171976963363056481941560715954676764349967630337 # equal to Pallas base field prime
GENESIS_BATCH = ""
EMPTY_NODE_HASH = ""
ORDER_FAIRNESS_THRESHOLD = 0.51

class Utils:
  @staticmethod
  def merkle_root_of_transaction_list(txs: list[TransactionId]) -> HashValue:
    if len(txs) == 0: return EMPTY_NODE_HASH
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

class Vote:
  head_batch_hash: HashValue

  def __init__(self, head_batch_hash: HashValue):
    self.head_batch_hash = head_batch_hash

  def __str__(self):
    return f"Vote(head_batch_hash={self.head_batch_hash})"

class BatchProposal:
  batch_hash: HashValue
  prev_batch_hash: HashValue
  deferred_ordering: List[TransactionId] # the deferred ordering of the third last head witness (if exists) from current witness backwards
  next_beacon_randomness: HashValue # peers use this to derive the leader of round r + BEACON_PACE
  solid_transactions: Set[TransactionId] # transactions that are received by n - f peers at this batch

  def __init__(self, round_number: int, prev_batch_hash: HashValue, deferred_ordering: List[TransactionId], prev_beacon_randomness: HashValue, solid_transactions: Set[TransactionId]):
    self.prev_batch_hash = prev_batch_hash
    self.deferred_ordering = deferred_ordering
    self.solid_transactions = solid_transactions
    self.round_number = round_number
    self.prev_beacon_randomness = prev_beacon_randomness
    self.next_beacon_randomness = self.compute_next_beacon_randomness()
    self.batch_hash = self.compute_batch_hash()
    # supporting data, can be locally inferred and don't have to be transferred over the network
    self.non_solid_txs: List[TransactionId] = []
    self.nodes_in_cone: Set[NodeId] = set()

  def store_nodes_in_cone(self, nodes: Set[NodeId]):
    self.nodes_in_cone = nodes

  def store_non_solid_txs(self, txs_list: List[TransactionId]):
    self.non_solid_txs = txs_list # store all the non solid transactions that aren't included in all the batch up to it
  
  def merkle_root_of_deferred_ordering(self) -> HashValue:
    return Utils.merkle_root_of_transaction_list(self.deferred_ordering)

  def merkle_root_of_solid_transactions(self) -> HashValue:
    return Utils.merkle_root_of_transaction_list(sorted(list(self.solid_transactions)))

  def compute_next_beacon_randomness(self) -> HashValue:
    components = [self.prev_beacon_randomness, str(self.round_number + BEACON_PACE), self.prev_batch_hash, self.merkle_root_of_deferred_ordering(), self.merkle_root_of_solid_transactions()]
    return hashlib.sha256(''.join(components).encode()).hexdigest()

  def compute_batch_hash(self) -> HashValue:
    # use hashlib of [prev_batch_hash, deferred_ordering, next_beacon_randomness]
    components = [str(self.round_number), self.prev_batch_hash, self.merkle_root_of_deferred_ordering(), self.merkle_root_of_solid_transactions(), self.next_beacon_randomness]
    return hashlib.sha256(''.join(components).encode()).hexdigest()[:8]

  def verify_batch_proposal_is_well_formed(self) -> bool:
    if not self.next_beacon_randomness == self.compute_next_beacon_randomness():
      return False
    if not self.batch_hash == self.compute_batch_hash():
      return False
    return True

  def clone(self):
    return BatchProposal(round_number=self.round_number, prev_batch_hash=self.prev_batch_hash, deferred_ordering=[tx for tx in self.deferred_ordering], prev_beacon_randomness=self.prev_beacon_randomness, solid_transactions=[tx for tx in self.solid_transactions])

  def __str__(self):
    return f"BatchProposal(batch_hash={self.batch_hash}, prev_batch_hash={self.prev_batch_hash}, prev_beacon_randomness={self.prev_beacon_randomness}, deferred_ordering={self.deferred_ordering}, next_beacon_randomness={self.next_beacon_randomness}, solid_transactions={self.solid_transactions}, non_solid_txs={self.non_solid_txs})"

class NodeMetadata:
  batch_proposal: BatchProposal # can be None if the node is not a head node (head node means the witness of the leader in its selected round)
  vote: Vote # BFT vote for the head batch of the network, which determines the order of transactions in the network
  creator_signature: Signature

  def __init__(self, batch_proposal: BatchProposal = None, vote: Vote = Vote(GENESIS_BATCH), creator_signature: Signature = ""):
    self.batch_proposal = batch_proposal
    self.vote = vote
    self.creator_signature = creator_signature

  def clone(self):
    return NodeMetadata(self.batch_proposal.clone() if self.batch_proposal else None, Vote(self.vote.head_batch_hash), self.creator_signature)

class Node:
    def __init__(self, peer_id: PeerId, height: int, round: int, is_witness: bool, newly_seen_txs_list: list[TransactionId], self_parent_hash: NodeId, cross_parent_hash: NodeId, metadata: NodeMetadata):
        self.peer_id = peer_id
        self.height = height
        self.is_witness = is_witness
        self.round = round
        self.self_parent_hash = self_parent_hash
        self.cross_parent_hash = cross_parent_hash
        # TODO: migrate to Sparse Merkle Tree + proof of SMT transition
        self.newly_seen_txs_list = newly_seen_txs_list
        self.node_hash = self.hash_node(peer_id, height, round, is_witness, self_parent_hash, cross_parent_hash, newly_seen_txs_list)
        ## fork-related data, must all be None until computed
        self.equivocated_peers: Set[PeerId] = None # set of peers that current node believes are equivocated, and this node won't SEE (i.e. UNSEE) all nodes created by them. Note that this doesn't affect STRONGLY SEEING property of this node.
        self.non_equivocated_peers: Set[PeerId] = None # set of peers that current node believes are not equivocated, and this node will SEE all nodes created by them.
        self.seen_nodes: Set[NodeId] = None # set of nodes that current node sees
        self.latest_seen_node_by_peers: Dict[PeerId, NodeId] = None # latest seen nodes from each peer that current node can see
        self.latest_seen_witness_by_peers: Dict[PeerId, NodeId] = None # set of witnesses of the previous round that current node can see
        self.seen_votes_by_peers: Dict[PeerId, HashValue] = None # latest votes from each peer that current node can see
        # metdata
        self.metadata: NodeMetadata = metadata
        self.update_signature()
        self.has_filled_node_data = False
  
    def clone(self):
      return Node(self.peer_id, self.height, self.round, self.is_witness, [txs for txs in self.newly_seen_txs_list], self.self_parent_hash, self.cross_parent_hash, self.metadata.clone())

    def label(self) -> NodeLabel:
      return f"{self.peer_id}:{self.node_hash}"
    
    def is_head_node(self) -> bool:
      return self.is_witness and self.metadata.batch_proposal is not None

    def compute_signature(self, _creator_private_key: str = None) -> Signature:
      # TODO: use real private key
      if self.metadata.batch_proposal:
        # signature = hash(batch_proposal.batch_hash, node_hash)
        return hashlib.sha256('#'.join([self.metadata.batch_proposal.batch_hash, self.node_hash]).encode()).hexdigest()[:8]
      else:
        # signature = hash(node_hash)
        return hashlib.sha256(self.node_hash.encode()).hexdigest()[:8]
    
    def update_signature(self):
      self.metadata.creator_signature = self.compute_signature()

    def verify_signature(self, creator_pubkey: Pubkey) -> bool:
      # TODO: use real pubkey
      return self.metadata.creator_signature == self.compute_signature()

    @staticmethod
    def hash_node(creator: PeerId, height: int, round: int, is_witness: bool, self_parent_hash: NodeId, cross_parent_hash: NodeId, newly_seen_txs_list: list[TransactionId]) -> NodeId:
        """Create deterministic hash for a node"""
        components = [creator, str(height), str(round), str(is_witness)]
        if cross_parent_hash:
            components.append(cross_parent_hash)
        if self_parent_hash:
            components.append(self_parent_hash)
        if newly_seen_txs_list:
            components.append(Utils.merkle_root_of_transaction_list(newly_seen_txs_list))
        return hashlib.sha256(''.join(components).encode()).hexdigest()[:8] # the hash value of a node basically depends deterministically on all of its content

    def verify_node_hash(self) -> bool:
      return self.node_hash == Node.hash_node(self.peer_id, self.height, self.round, self.is_witness, self.self_parent_hash, self.cross_parent_hash, self.newly_seen_txs_list)

    def validate_node_data(self, creator_pubkey: Pubkey) -> bool:
      if not self.verify_node_hash():
        return False
      
      # TODO: validate that all the node data is well-formed (use schema validator)
      
      if self.metadata.batch_proposal:
        if not self.metadata.batch_proposal.verify_batch_proposal_is_well_formed():
          return False
        # it must vote for its own batch proposal
        if self.metadata.vote.head_batch_hash != self.metadata.batch_proposal.batch_hash:
          return False
      
      if not self.verify_signature(creator_pubkey):
        return False
 
      return True

    def is_genesis(self):
      is_genesis = self.round == 0 and self.is_witness == True and self.self_parent_hash == "" and self.cross_parent_hash == "" and self.verify_node_hash()
      return is_genesis

    def is_non_genesis(self):
      return not self.is_genesis()

    def get_seen_valid_peers(self) -> Set[PeerId]:
      eq

    def __str__(self):
      batch_info = ("batch_hash=" + self.metadata.batch_proposal.__str__() if self.metadata.batch_proposal else "")
      vote_info = ("vote=" + self.metadata.vote.__str__() if self.metadata.vote else "none")
      return f"{"GENESIS " if self.is_genesis() else ""}Node(node_hash={self.node_hash}, peer_id={self.peer_id}, height={self.height}, round={self.round}, is_witness={self.is_witness}, self_parent_hash={self.self_parent_hash}, cross_parent_hash={self.cross_parent_hash}, newly_seen_txs_list={self.newly_seen_txs_list}, {batch_info}, {vote_info})" # , equivocated_peers={self.equivocated_peers}, seen_nodes={self.seen_nodes})"

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
        self.should_exit = False

    def N(self) -> int:
      return len(self.peers)
    
    def f(self) -> int:
      return ((self.N() - 1) // 3)

    def safe_threshold(self) -> int:
      return (self.N() - self.f())

    # TODO: use secure cryptographic randomness source
    def get_first_beacon_randomness(self) -> HashValue:
      all_peers = [p.peer_id for p in self.peers]
      return hashlib.sha256(",".join(all_peers).encode()).hexdigest()

    def get_first_leaders(self) -> List[PeerId]: # for round 1 -> BEACON_PACE
      all_peers = [p.peer_id for p in self.peers]
      first_beacon_randomness = self.get_first_beacon_randomness()
      # choose the first BEACON_PACE leaders for the first BEACON_PACE rounds, using the first BEACON_FIELD_PRIME as modulo somehow
      beacon_random = random.Random(first_beacon_randomness)
      leaders = []
      for i in range(BEACON_PACE):
        leaders.append(beacon_random.choice(all_peers))
      return leaders

    def get_leader_after_BEACON_PACE_rounds(self, beacon_randomness: HashValue) -> PeerId:
      beacon_random = random.Random(beacon_randomness)
      all_peers = [p.peer_id for p in self.peers]
      return beacon_random.choice(all_peers)

    def get_peer_pubkey(self, peer_id: PeerId) -> Pubkey:
      # TODO: use real pubkey
      return "DUMMY_PUBKEY"

    def register_peer(self, peer: 'ConsensusPeer'):
        # peer_id must be unique
        (peer.peer_id not in [p.peer_id for p in self.peers]) or (_ for _ in ()).throw(ValueError("peer_id must be unique"))
        self.peers.append(peer)

    def get_all_peer_ids(self) -> List[PeerId]:
        return [p.peer_id for p in self.peers]

    def unregister_peer(self, peer: 'ConsensusPeer'):
        (peer in self.peers) or (_ for _ in ()).throw(ValueError("peer must be registered"))
        self.peers.remove(peer)

    async def register_genesis_nodes(self):
        """Register genesis nodes from all peers as the first checkpoint"""
        # Create genesis nodes and initial gossip
        genesis_nodes: Dict[PeerId, Node] = {}
        for peer1 in self.peers:
            genesis_node = peer1.create_genesis_node()
            peer1.construct_batch_proposal_if_needed(genesis_node)

            genesis_nodes[peer1.peer_id] = genesis_node
            for peer2 in self.peers:
              if peer2.peer_id == peer1.peer_id or peer1.peer_id in peer2.equivocated_peers or peer2.peer_id in peer1.equivocated_peers:
                continue

              cloned_genesis_node = genesis_node.clone() # simulate the process of serializing and deserializing the nodes in internet protocols

              # Gossip genesis node to neighbors
              success = await self.gossip_send_node_and_ancestry(peer1.peer_id, peer2.peer_id, cloned_genesis_node)
              if not success:
                print(f"peer {peer1.peer_id} gossiped to {peer2.peer_id} genesis node {genesis_node.node_hash} failed")
              else:
                print(f"peer {peer1.peer_id} gossiped to {peer2.peer_id} genesis node {genesis_node.node_hash} successfully")

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
          if len(mempool_txs) > 0 and self.random_instance.random() < 0.1:
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
        return True
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
        sender != receiver or (_ for _ in ()).throw(ValueError("sender and receiver must be different"))

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

        if sender in receiver_peer.equivocated_peers:
          return False

        if receiver_peer.has_seen_valid_node(node1):
          return True
  
        all_received_nodes: List[Node] = []
        traced: Dict[NodeId, bool] = {}

        def trace(node: Node):
          if node.node_hash in traced or receiver_peer.has_seen_valid_node(node):
            return
          traced[node.node_hash] = True

          predecessors = sender_peer.get_predecessors(node)
          for predecessor in predecessors:
            trace(predecessor)

          all_received_nodes.append(node)

        trace(node1)

        for i in range(len(all_received_nodes)):
          current_node = all_received_nodes[i]
          try:
            is_success = receiver_peer.verify_node_and_add_to_local_view(current_node.clone(), sender=sender_peer.peer_id) or (_ for _ in ()).throw(ValueError("failed to verify and add node"))
            (is_success == True) or (_ for _ in ()).throw(ValueError("failed to verify and add node"))
          except Exception as e:
            # if sender_peer.peer_id not in receiver_peer.equivocated_peers:
            ddebug(receiver_peer.peer_id, f"(R={receiver_peer.peer_id},S={sender_peer.peer_id}) rejected node {current_node.node_hash} created by {current_node.peer_id}: {e}")
            isWrong = not sender_peer.is_adversary and not receiver_peer.is_adversary
            if isWrong:
              ddebug(receiver_peer.peer_id, f"e = {e}")
              ddebug(receiver_peer.peer_id, f"INVALID REJECTION of {current_node.node_hash} with (sender={sender_peer.peer_id}, receiver={receiver_peer.peer_id})")
              self.should_exit = True
              break
            continue

          print(f"(R={receiver_peer.peer_id},S={sender_peer.peer_id}) accepted node {current_node.node_hash} created by {current_node.peer_id}, {current_node}")

        return receiver_peer.has_seen_valid_node(node1)

    async def _delay(self):
        """Simulate network latency"""
        delay = 0
        # TODO: turn on delay (a value in the self.latency_range range) for full simulation
        # await asyncio.sleep(delay)
        pass
      
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
        self.pos_in_seen_valid_nodes: Dict[NodeId, (PeerId, int)] = {} # maps from node_hash to (peer_id, position) of the node in self.seen_valid_nodes[peer_id]
        
        self.accumulated_txs = set()  # Track all transactions seen by this peer
        self.network: NetworkSimulator = network
        ## adversary-related data
        self.equivocation_prob = 0.3 if is_adversary else 0.0
        self.equivocated_peers: Set[PeerId] = set()
        self.neighbors: list[PeerId] = []  # Track neighboring peers
        # if each peer connects to log(N) neighbors, a transaction would takes O(log(N)/log(log(N))) gossip hops to reach the whole network
        # for N = 10^6, it would be 7 hops
        self.tx_receive_times = {}  # Track when transactions were received
        self.pending_txs: List[Tuple[TransactionId, float]] = []
        self.local_graph: Dict[NodeId, Set[NodeId]] = {} # node_hash to set of node_hashes (parent, child)
        # batch-related data
        self.observed_valid_batches: Dict[HashValue, Node] = {} # map from the hash value of the batch to the node that proposes it
        self.first_inclusion_of_txs_at_peer: Dict[PeerId, Dict[TransactionId, NodeId]] = {} # map from peer_id to map from tx_id to the first time it is included at that peer
        self.cached_heaviest_batch_amongst_strict_ancestors: Dict[NodeId, HashValue] = {}
        self.cached_head_node: Dict[NodeId, bool] = {}

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
      (self_parent_node is not None and cross_parent_node is not None) or (_ for _ in ()).throw(ValueError("self_parent_node and cross_parent_node must be non-None"))
      predecessors.append(self_parent_node)
      predecessors.append(cross_parent_node)
      return predecessors
    
    def get_successors(self, node: Node) -> list[Node]:
      successors = []
      for successor in self.local_graph.get(node.node_hash, set()):
        successors.append(self.get_node_by_hash(successor))
      return sorted(successors, key=lambda x: x.round)

    def get_graph_info(self) -> Dict[NodeId, Node]:
      res: Dict[NodeId, Node] = {}
      for peer_id in self.seen_valid_nodes:
        for node in self.seen_valid_nodes[peer_id]:
          res[node.node_hash] = node

      return res

    def visualize_view(self):
      # print(f"peer {self.peer_id} sees the DAG:")
      # for peer_id in self.seen_valid_nodes:
      #   for node in self.seen_valid_nodes[peer_id]:
      #     print(f"{node}")

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
          
          node_positions[node.node_hash] = (max(node_positions[predecessor.node_hash][0] + 10, node_positions[node.node_hash][0]), line_id)

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
              node_size=500,
              node_shape='s',
              node_color=node_color,
              label=[node.peer_id]
          )

      nx.draw_networkx_edges(G, node_positions, edge_color='black', arrows=True, arrowsize=20, width=1, node_size=500)  # Ensure arrows are properly sized relative to nodes

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
            height=0,
            round=0,
            is_witness=True,
            newly_seen_txs_list=[],
            self_parent_hash=EMPTY_NODE_HASH,
            cross_parent_hash=EMPTY_NODE_HASH,
            metadata=NodeMetadata()
        )
        (self.verify_node_and_add_to_local_view(node, sender=self.peer_id) == True) or (_ for _ in ()).throw(ValueError("failed to verify and add genesis node"))
        return node

    def get_heaviest_batch_amongst_strict_ancestors(self, node: Node) -> HashValue:
      """
      Get the heaviest batch in the strict ancestors of the given node using a fork-choice rule based on the Heaviest Observed Subtree (HOS) selection rule
      """
      if node.node_hash in self.cached_heaviest_batch_amongst_strict_ancestors:
        return self.cached_heaviest_batch_amongst_strict_ancestors[node.node_hash]

      # 1. construct a tree of batch proposals
      tree: Dict[HashValue, Set[HashValue]] = {GENESIS_BATCH: set()}

      valid_batches = self.observed_valid_batches.values()
      valid_batches = [cur_node for cur_node in valid_batches if self.is_seen_by(cur_node.node_hash, node)]

      for node in valid_batches: # TODO: only use the batches from the last finalized branch, so we can filter out the batches from equivocated peers
        tree[node.metadata.batch_proposal.batch_hash] = set()
        parent_batch_hash = node.metadata.batch_proposal.prev_batch_hash
        if parent_batch_hash not in tree:
          tree[parent_batch_hash] = set()
        tree[parent_batch_hash].add(node.metadata.batch_proposal.batch_hash)
      # 2. collect the latest votes of all peers that the node can see
      weight_of_branch: Dict[HashValue, int] = {GENESIS_BATCH: 0}

      non_equivocated_peers = node.non_equivocated_peers
      for peer_id in node.seen_votes_by_peers:
        voted_batch_hash = node.seen_votes_by_peers[peer_id]
        (voted_batch_hash in tree) or (_ for _ in ()).throw(ValueError("voted_batch_hash must be in tree"))
        weight_of_branch[voted_batch_hash] += 1
      # 3. accumulate weights upwards from the leaves to the root
      def traverse(batch_hash: HashValue):
        weight_of_branch[batch_hash] = 0
        for child_batch_hash in tree[batch_hash]:
          traverse(child_batch_hash)
          weight_of_branch[batch_hash] += weight_of_branch[child_batch_hash]
      root = GENESIS_BATCH
      traverse(root)
      # 4. traverse downwards from the root to leaves using the HOS rule
      heaviest_batch = GENESIS_BATCH
      while heaviest_batch in tree:
        if len(tree[heaviest_batch]) == 0:
          break
        heaviest_batch = max(tree[heaviest_batch], key=lambda x: weight_of_branch[x])

      self.cached_heaviest_batch_amongst_strict_ancestors[node.node_hash] = heaviest_batch
      return heaviest_batch

    def get_beacon_randomness_of_batch(self, batch_hash: HashValue) -> HashValue:
      """
      Get the beacon randomness of the given batch
      """
      if batch_hash == GENESIS_BATCH:
        return self.network.get_first_beacon_randomness()
      else:
        return self.observed_valid_batches[batch_hash].metadata.batch_proposal.next_beacon_randomness

    def get_node_of_batch(self, batch_hash: HashValue) -> Node:
      """
      Get the node of the given batch
      """
      if batch_hash == GENESIS_BATCH:
        return None
      else:
        return self.observed_valid_batches[batch_hash]

    def verify_node_is_head_node(self, node: Node) -> bool:
      """
      Verify that the given node is a head node, given that the vote is valid
      """

      if node.node_hash in self.cached_head_node:
        return self.cached_head_node[node.node_hash]
      
      if not node.is_witness or node.is_genesis(): # this function might be called before the batch proposal is constructed so we only needs to check whether the node is a witness
        return False

      heaviest_batch_amongst_strict_ancestors = self.get_heaviest_batch_amongst_strict_ancestors(node)

      # find the batch that contains the randomness that determines the leader of the current round
      beacon_batch:HashValue = heaviest_batch_amongst_strict_ancestors
      steps_back = BEACON_PACE

      for i in range(steps_back - 1):
        if beacon_batch == GENESIS_BATCH:
          break
        # find parent batch of the current beacon batch
        beacon_batch = self.observed_valid_batches[beacon_batch].metadata.batch_proposal.prev_batch_hash


      beacon_randomness = f"{0 if beacon_batch == GENESIS_BATCH else self.get_beacon_randomness_of_batch(beacon_batch)}:{node.round}"
      hash_to_big_int = int(hashlib.sha256(beacon_randomness.encode()).hexdigest(), 16)

      all_peers = sorted([p.peer_id for p in self.network.peers])
      selected_peer = all_peers[hash_to_big_int % len(all_peers)]
      
      if node.round >= 1:
        if selected_peer == node.peer_id:
          if self.peer_id == node.peer_id:
            ddebug(self.peer_id, f" {node.node_hash} found leader of round {node.round}: beacon_batch = {beacon_batch}, beacon_randomness = {beacon_randomness} => {selected_peer}")
      
      self.cached_head_node[node.node_hash] = selected_peer == node.peer_id
      return self.cached_head_node[node.node_hash]

      # compute the leader of the current round based on the beacon randomness
      
      # if beacon_batch == GENESIS_BATCH:
      #   first_leaders: List[PeerId] = self.network.get_first_leaders()
      #   is_head_node = len(first_leaders) == BEACON_PACE and node.round > 0 and node.round <= BEACON_PACE and node.peer_id == first_leaders[node.round - 1]
      #   return is_head_node
      # else:
      #   # non-genesis beacon randomness
      #   return node.peer_id == self.network.get_leader_after_BEACON_PACE_rounds(self.get_beacon_randomness_of_batch(beacon_batch))

    def is_seen_by(self, node_id: NodeId, dest_node: Node) -> bool:
      if dest_node == None:
        return False

      if dest_node.node_hash == node_id:
        return True

      for mid_node_hash in dest_node.latest_seen_node_by_peers.values():
        if self.is_valid_descendant_and_self_ancestor(mid_node_hash, node_id):
          return True

      return False

    def check_transaction_included_up_to_batch(self, tx: TransactionId, batch_hash: HashValue) -> bool:
      # TODO: use efficient lookup strategy
      while batch_hash != GENESIS_BATCH:
        if tx in self.observed_valid_batches[batch_hash].metadata.batch_proposal.solid_transactions:
          return True
        batch_hash = self.observed_valid_batches[batch_hash].metadata.batch_proposal.prev_batch_hash
      return False


    def map_to_cone_region(self, first_inclusion_location: NodeId, left_boundary_node: Node, head_witness: Node) -> NodeId:
      """
      Map the first inclusion location of a transaction to the cone region of the given head witness
      """
      if first_inclusion_location == EMPTY_NODE_HASH:
        return EMPTY_NODE_HASH

      # check if the first inclusion location is seen by head_witness but not left_boundary_node
      if self.is_seen_by(first_inclusion_location, head_witness) and not self.is_seen_by(first_inclusion_location, left_boundary_node):
        return first_inclusion_location
      
      return EMPTY_NODE_HASH
    
    def print_final_transaction_order(self) -> List[TransactionId]:
      res: List[TransactionId] = []
      heaviest_batch = self.get_heaviest_batch_amongst_strict_ancestors(self.get_my_last_node())

      while heaviest_batch != GENESIS_BATCH:
        cur_witness: Node = self.get_node_of_batch(heaviest_batch)
        res.extend(cur_witness.metadata.batch_proposal.deferred_ordering[::-1])
        heaviest_batch = self.observed_valid_batches[heaviest_batch].metadata.batch_proposal.prev_batch_hash
      return res[::-1]

    def construct_semi_complete_digraph_of_transactions(self, last_3_head_witnesses: List[Node]) -> SemiCompleteDiGraph:
      (len(last_3_head_witnesses) == 3) or (_ for _ in ()).throw(ValueError("last_3_head_witnesses must have length 3"))

      if any(head_witness is None for head_witness in last_3_head_witnesses):
        return SemiCompleteDiGraph()

      dest_node = last_3_head_witnesses[2]
      rounds: List[int] = [node.round for node in last_3_head_witnesses]
      ordered_head_witness: Node = last_3_head_witnesses[0]
      left_boundary_node: Node = self.get_node_of_batch(ordered_head_witness.metadata.batch_proposal.prev_batch_hash)
      solid_transactions: Set[TransactionId] = ordered_head_witness.metadata.batch_proposal.solid_transactions
      dependency_graph: SemiCompleteDiGraph = SemiCompleteDiGraph()
      for u in solid_transactions:
        dependency_graph.add_node(u)

      traced: Dict[NodeId, bool] = {}
      nodes_by_peer_id: Dict[PeerId, Node] = {}

      all_legitimate_peers = [peer_id for peer_id in self.network.get_all_peer_ids() if peer_id not in dest_node.equivocated_peers] # TODO: handle the case that the list of peers changes

      preference_graphs: Dict[PeerId, List[TransactionId]] = {}
      for peer_id in all_legitimate_peers:
        preference_graphs[peer_id] = []

      solid_edge: Set[Tuple[TransactionId, TransactionId]] = set()
      for tx1 in solid_transactions:
        for tx2 in solid_transactions:
          if tx1 < tx2:
            count_receipts_of_edge_at_batches: List[int] = [0] * len(last_3_head_witnesses)
            count_opposite_receipts_of_edge_at_batches: List[int] = [0] * len(last_3_head_witnesses)

            for i in range(len(last_3_head_witnesses)):
              head_witness: Node = last_3_head_witnesses[i]
              for peer_id in all_legitimate_peers:
                if peer_id not in self.first_inclusion_of_txs_at_peer: continue
                first_inclusion_of_tx1_at_peer = self.first_inclusion_of_txs_at_peer[peer_id].get(tx1, EMPTY_NODE_HASH)
                first_inclusion_of_tx2_at_peer = self.first_inclusion_of_txs_at_peer[peer_id].get(tx2, EMPTY_NODE_HASH)
                # map the first inclusion locations into the cone region of the current head witness
                mapped_first_inclusion_of_tx1_at_peer = self.map_to_cone_region(first_inclusion_of_tx1_at_peer, left_boundary_node, head_witness)
                mapped_first_inclusion_of_tx2_at_peer = self.map_to_cone_region(first_inclusion_of_tx2_at_peer, left_boundary_node, head_witness)

                is_left_seen = mapped_first_inclusion_of_tx1_at_peer != EMPTY_NODE_HASH
                is_right_seen = mapped_first_inclusion_of_tx2_at_peer != EMPTY_NODE_HASH

                if is_left_seen:
                  if not is_right_seen: # (tx1 -> tx2)
                      count_opposite_receipts_of_edge_at_batches[i] += 1
                else:
                  if is_right_seen: # (tx2 -> tx1)
                      count_receipts_of_edge_at_batches[i] += 1
                
                if is_left_seen and is_right_seen:
                  if self.is_seen_by(mapped_first_inclusion_of_tx1_at_peer, self.get_node_by_hash(mapped_first_inclusion_of_tx2_at_peer)):  # (tx1 -> tx2)
                    count_receipts_of_edge_at_batches[i] += 1
                  else: # (tx2 -> tx1)
                    count_opposite_receipts_of_edge_at_batches[i] += 1

              # solid edge
              if count_receipts_of_edge_at_batches[2] >= ORDER_FAIRNESS_THRESHOLD * len(all_legitimate_peers):
                dependency_graph.add_directed_edge(tx1, tx2)
                solid_edge.add((tx1, tx2))
              elif count_opposite_receipts_of_edge_at_batches[2] >= ORDER_FAIRNESS_THRESHOLD * len(all_legitimate_peers):
                dependency_graph.add_directed_edge(tx2, tx1)
              else:
                # soft edge
                for i in range(len(last_3_head_witnesses)):
                  if count_receipts_of_edge_at_batches[i] >= count_opposite_receipts_of_edge_at_batches[i]:
                    dependency_graph.add_directed_edge(tx1, tx2)
                  elif count_receipts_of_edge_at_batches[i] <= count_opposite_receipts_of_edge_at_batches[i]:
                    dependency_graph.add_directed_edge(tx2, tx1)

      dependency_graph.assert_is_semi_complete_digraph() # there must be at least one edge between any two transactions
      
      return dependency_graph

    def get_solid_transactions_and_non_solid_transactions_in_cone(self, head_node: Node, prev_batch_hash: HashValue) -> Tuple[List[TransactionId], List[TransactionId], List[NodeId]]:
      """
      Get the solid transactions in the truncated cone of the given head node and previous batch hash
      """
      traced: Dict[NodeId, bool] = {}
      current_cone: List[Node] = []

      prev_head_witness = self.get_node_of_batch(prev_batch_hash)

      unincluded_solid_txs_from_prev_batch: Set[TransactionId] = set()

      if prev_head_witness:
        unincluded_solid_txs_from_prev_batch.update(prev_head_witness.metadata.batch_proposal.non_solid_txs) # it now becomes solid due to strongly-seeing property (i.e. n - f peers each of these transactions)

      # TODO: use more lightweight data structure here
      all_prev_solid_txs: Set[TransactionId] = set()
      all_prev_cone_nodes: Set[NodeId] = set()
      
      cur_batch = prev_batch_hash
      while cur_batch != GENESIS_BATCH:
        cur_head_witness = self.observed_valid_batches[cur_batch]
        all_prev_solid_txs.update(cur_head_witness.metadata.batch_proposal.solid_transactions)
        all_prev_cone_nodes.update(cur_head_witness.metadata.batch_proposal.nodes_in_cone)
        cur_batch = cur_head_witness.metadata.batch_proposal.prev_batch_hash
      
      def find_node_in_current_cone(node: Node):
        if node.node_hash in traced:
          return
        
        if node.node_hash in all_prev_cone_nodes:
          return

        traced[node.node_hash] = True

        predecessors = self.get_predecessors(node)
        for predecessor in predecessors:
          find_node_in_current_cone(predecessor)

        current_cone.append(node)

      find_node_in_current_cone(head_node)

      # filter & keep only the transactions that aren't included in previous batches and received by n - f legitimate peers
      count_legitimate_receipts_of_tx: Dict[TransactionId, int] = {}
      for node in current_cone:
        if node.peer_id in head_node.equivocated_peers: # ignore opinions of equivocated peers
          # TODO: only ignore the suffix from which forks are detected
          continue

        for tx in node.newly_seen_txs_list:
          count_legitimate_receipts_of_tx[tx] = count_legitimate_receipts_of_tx.get(tx, 0) + 1

      n = len(self.network.get_all_peer_ids())
      f = (n - 1)/3

      new_candidate_solid_transactions: Set[TransactionId] = set()
      new_pending_txs: List[TransactionId] = []

      for tx in count_legitimate_receipts_of_tx:
        if tx not in all_prev_solid_txs and tx not in unincluded_solid_txs_from_prev_batch:
          if count_legitimate_receipts_of_tx[tx] >= n - f:
            new_candidate_solid_transactions.add(tx)
          else:
            new_pending_txs.append(tx)
      
      solid_transactions: List[TransactionId] = sorted(list(new_candidate_solid_transactions.union(unincluded_solid_txs_from_prev_batch)))
      pending_txs: List[TransactionId] = sorted(new_pending_txs)

      return solid_transactions, pending_txs, [node.node_hash for node in current_cone]

    def get_deferred_ordering_of_transactions(self, last_3_head_witnesses: List[Node]) -> List[TransactionId]:
      """
      Get the deferred ordering of the transactions in the truncated cone of the given head node and previous batch hash
      """
      # gather all transactions in the truncated cone into a tournament graph
      dependency_graph: SemiCompleteDiGraph = self.construct_semi_complete_digraph_of_transactions(last_3_head_witnesses)
      # calculate the fair ordering of the transactions and construct the batch proposal
      sccs = dependency_graph.find_strongly_connected_components()
      sccs_with_fair_orderings = [[scc, dependency_graph.find_hamiltonian_path(scc[0])] for scc in sccs]
      deferred_ordering: List[TransactionId] = []
      for scc, fair_ordering in sccs_with_fair_orderings:
        deferred_ordering.extend([TransactionId(tx) for tx in fair_ordering])

      assert len(deferred_ordering) == len(dependency_graph.nodes)
      print(f"Peer {self.peer_id} got deferred ordering for node_hash = {last_3_head_witnesses[2].node_hash} = {deferred_ordering}")

      return deferred_ordering

    def compute_batch_proposal(self, dest_node: Node) -> BatchProposal:
      """
      Compute a batch proposal for the given node
      """
      # fork-choice rule: select the previous batch using the Heaviest Observed Subtree (HOS) selection rule

      
      heaviest_batch: HashValue = self.get_heaviest_batch_amongst_strict_ancestors(dest_node)
      prev_beacon_randomness: HashValue = self.get_beacon_randomness_of_batch(heaviest_batch)

      ## find the last 3 head witnesses
      last_3_head_witnesses: List[Node] = [dest_node]
      cur_batch_hash = heaviest_batch
      for i in range(2):
        if cur_batch_hash == GENESIS_BATCH:
          last_3_head_witnesses.append(None)
        else:
          prev_head_witness = self.observed_valid_batches[cur_batch_hash] if cur_batch_hash in self.observed_valid_batches else None
          (prev_head_witness is not None) or (_ for _ in ()).throw(ValueError("prev_head_witness must be non-None"))
          last_3_head_witnesses.append(prev_head_witness)
          cur_batch_hash = prev_head_witness.metadata.batch_proposal.prev_batch_hash
      last_3_head_witnesses.reverse()

      # construct the truncated cone of round (r)
      deferred_ordering: List[TransactionId] = []
      solid_transactions, non_solid_transactions, nodes_in_cone = self.get_solid_transactions_and_non_solid_transactions_in_cone(dest_node, heaviest_batch)

      if len(last_3_head_witnesses) >= 3:
        deferred_ordering = self.get_deferred_ordering_of_transactions(last_3_head_witnesses)

      batch_proposal = BatchProposal(
        round_number=dest_node.round,
        prev_batch_hash=heaviest_batch,
        deferred_ordering=deferred_ordering,
        prev_beacon_randomness=prev_beacon_randomness,
        solid_transactions=solid_transactions
      )

      batch_proposal.store_non_solid_txs(non_solid_transactions)
      batch_proposal.store_nodes_in_cone(nodes_in_cone)

      (batch_proposal.verify_batch_proposal_is_well_formed()) or (_ for _ in ()).throw(ValueError("batch proposal is not well-formed"))

      return batch_proposal

    def construct_batch_proposal_if_needed(self, node: Node):
      """
      Construct a batch proposal for the given node. This must be called after the vote is constructed for the node.
      """
      if not self.verify_node_is_head_node(node):
        return
      
      batch_proposal: BatchProposal = self.compute_batch_proposal(node)

      node.metadata = NodeMetadata(batch_proposal=batch_proposal)

    # TODO: implement bootstrap node (first node refers to parents in a checkpoint after a node rejoins the network)
    
    def verify_batch_proposal_is_valid(self, node: Node) -> bool:
      """
      Verify that the given batch proposal is valid
      """
      (node.metadata.batch_proposal is not None) or (_ for _ in ()).throw(ValueError("batch proposal must be non-None"))
     
      correct_batch_proposal: BatchProposal = self.compute_batch_proposal(node)
      
      return node.metadata.batch_proposal.batch_hash == correct_batch_proposal.batch_hash

    def compute_vote_for_node(self, node: Node) -> Vote:
      """
      Compute a vote for the given node
      """
      is_head_node = self.verify_node_is_head_node(node)

      if is_head_node:
        return Vote(head_batch_hash=node.metadata.batch_proposal.batch_hash) # vote for its own head batch
      else:
        return Vote(head_batch_hash=self.get_heaviest_batch_amongst_strict_ancestors(node)) # vote for the heaviest batch amongst its strict ancestors

    def construct_vote_for_node(self, node: Node):
      """
      Construct a vote for the given node
      """
      vote = self.compute_vote_for_node(node)
      node.metadata.vote = vote

    def verify_vote_is_valid(self, node: Node) -> bool:
      """
      Verify that the given vote is valid
      """
      return node.metadata.vote.head_batch_hash == self.compute_vote_for_node(node).head_batch_hash

    def fill_node_data(self, node: Node):
      """
      Fill the data for the given node
      """
      if not node.has_filled_node_data:
        try:
          self.compute_seen_nodes_of_new_node(node)
          self.construct_batch_proposal_if_needed(node)
          self.construct_vote_for_node(node) # this will throw errors if predecessors of the node are not available
          node.has_filled_node_data = True
          node.update_signature()
        except Exception as e:
          # error when filling data for the node
          raise e
      
    def has_seen_valid_node(self, node: Node) -> bool:
      return node.node_hash in self.pos_in_seen_valid_nodes

    def get_node_by_hash(self, node_hash: NodeId) -> Optional[Node]:
      if node_hash in self.pos_in_seen_valid_nodes:
        peer_id, pos = self.pos_in_seen_valid_nodes[node_hash]
        return self.seen_valid_nodes[peer_id][pos]
      else:
        return None

    def record_transaction_receipt(self, tx_id: TransactionId, timestamp: float):
        """Record when a transaction was received"""
        self.tx_receive_times[tx_id] = timestamp
        self.pending_txs.append((tx_id, timestamp))

    def select_neighbors(self, all_peers: List[PeerId]):
        """Randomly select neighbors from available peers"""
        potential_neighbors = [p for p in all_peers if p != self.peer_id]
        num_neighbors = min(self.get_max_num_neighbors(), len(all_peers) - 1)
        self.neighbors = [peer_id for peer_id in self.random_instance.sample(potential_neighbors, num_neighbors) if peer_id not in self.equivocated_peers]

    def is_valid_descendant_and_self_ancestor(self, descendant_node_hash: NodeId, self_ancestor_node_hash: NodeId) -> bool:
      ancestor_node = self.get_node_by_hash(self_ancestor_node_hash)
      descendant_node = self.get_node_by_hash(descendant_node_hash)
      if ancestor_node is None or descendant_node is None:
        return False
      
      # TODO: optimize this to jump bigger steps for faster descendant check
      while descendant_node is not None and descendant_node.round >= ancestor_node.round:
        if descendant_node.node_hash == ancestor_node.node_hash:
          return True
        descendant_node = self.get_node_by_hash(descendant_node.self_parent_hash)
      return False
    
    def get_self_descendant(self, node_hash_1: NodeId, node_hash_2: NodeId) -> Optional[NodeId]:
      try:
        node1 = self.get_node_by_hash(node_hash_1)
        node2 = self.get_node_by_hash(node_hash_2)

        if not node1 or not node2 or node1.peer_id != node2.peer_id:
          return None

        res = node1 if node1.height > node2.height else node2

        while node1.height > node2.height:
          node1 = self.get_node_by_hash(node1.self_parent_hash)
          if not node1:
            return None

        while node2.height > node1.height:
          node2 = self.get_node_by_hash(node2.self_parent_hash)
          if not node2:
            return None

        if node1.node_hash == node2.node_hash:
          return res.node_hash
        else:
          return None
      except:
        return None
        
    def compute_seen_nodes_of_new_node(self, dest_node: Node):
        """Compute seen data of the new node by aggregating from parents and itself."""
        (dest_node.latest_seen_node_by_peers is None or
        dest_node.non_equivocated_peers is None or
        dest_node.equivocated_peers is None or
        dest_node.seen_votes_by_peers is None or
        dest_node.latest_seen_witness_by_peers is None) or (_ for _ in ()).throw(ValueError("Node must not have precomputed values."))

        latest_seen_node_by_peers: Dict[PeerId, NodeId] = {}
        non_equivocated_peers: Set[PeerId] = set()        
        equivocated_peers: Set[PeerId] = set()
        seen_votes_by_peers: Dict[PeerId, HashValue] = {}
        latest_seen_witness_by_peers: Dict[PeerId, NodeId] = {}

        aggregated_references = [dest_node.self_parent_hash, dest_node.cross_parent_hash]
        aggregated_references = [ref for ref in aggregated_references if ref is not EMPTY_NODE_HASH]

        def aggregate_latest_seen_by_peers(
          dest_dict: Dict[PeerId, NodeId],
          parent_dict: Dict[PeerId, NodeId],
          equivocated_peers: Set[PeerId],
        ):
          for peer_id, seen_node_hash in parent_dict.items():
            seen_node = self.get_node_by_hash(seen_node_hash)
            seen_node is not None or (_ for _ in ()).throw(ValueError("seen node must not be None"))

            existing_seen_node_hash = dest_dict.get(peer_id)
            if existing_seen_node_hash is not None:
                self_descendant_node_id = self.get_self_descendant(existing_seen_node_hash, seen_node_hash)
                if self_descendant_node_id is not None:
                    dest_dict[peer_id] = self_descendant_node_id
                else:
                    # two nodes form forks
                    equivocated_peers.add(peer_id)
            else:
                dest_dict[peer_id] = seen_node_hash
        
        # Aggregate data from parents
        for parent_hash in aggregated_references:
            if parent_hash == EMPTY_NODE_HASH: continue
            parent_node = self.get_node_by_hash(parent_hash)
            if parent_node is None:
              raise ValueError(f"Parent node {parent_hash} is None")
            
            latest_seen_by_peers_of_parent = parent_node.latest_seen_node_by_peers
            latest_seen_witness_by_peers_of_parent = parent_node.latest_seen_witness_by_peers

            aggregate_latest_seen_by_peers(
              dest_dict=latest_seen_node_by_peers,
              parent_dict=latest_seen_by_peers_of_parent,
              equivocated_peers=equivocated_peers
            )
            aggregate_latest_seen_by_peers(
              dest_dict=latest_seen_witness_by_peers,
              parent_dict=latest_seen_witness_by_peers_of_parent,
              equivocated_peers=equivocated_peers
            )
            
            # print(f"Peer {self.peer_id} is merging {parent_node} with {parent_node.equivocated_peers}")
            equivocated_peers.update(parent_node.equivocated_peers)
            seen_votes_by_peers.update(parent_node.seen_votes_by_peers) # the vote inside the dest_node would be updated later when verifying the vote of the dest_node

        for peer_id in equivocated_peers:
          if peer_id in latest_seen_node_by_peers:
            latest_seen_node_by_peers.pop(peer_id)
          if peer_id in latest_seen_witness_by_peers:
            latest_seen_witness_by_peers.pop(peer_id)
        
        for peer_id in latest_seen_node_by_peers.keys():        
          non_equivocated_peers.add(peer_id)
        # store the computed values

        dest_node.non_equivocated_peers = non_equivocated_peers
        dest_node.seen_votes_by_peers = seen_votes_by_peers
        dest_node.equivocated_peers = equivocated_peers
        # we only updates a node sees itself when it is verified and added to the local view in self.verify_node_and_add_to_local_view()
        dest_node.latest_seen_node_by_peers = latest_seen_node_by_peers

        dest_node.latest_seen_witness_by_peers = latest_seen_witness_by_peers
        ###
        self.equivocated_peers.update(dest_node.equivocated_peers)

    def verify_node_and_add_to_local_view(self, node: Node, sender: PeerId = None) -> bool:
        """Verify a node and its transactions, and add it to the local view"""
        ### Verification
        if sender in self.equivocated_peers and sender != self.peer_id:
          return False
        
        assert node is not None

        if self.has_seen_valid_node(node):
          return True

        self.fill_node_data(node)
        
        is_valid = self.verify_node(node)

        if not is_valid:
          return False
        
        if node.is_witness and node.metadata.batch_proposal is not None:
          print(f"Peer {self.peer_id} confirms node ({node.node_hash}, {node.peer_id}) is head node of round {node.round}")

        for tx in node.newly_seen_txs_list:
          if tx in self.first_inclusion_of_txs_at_peer[node.peer_id]: # node.peer_id must be equivocated
            self.equivocated_peers.add(node.peer_id)
            if sender == node.peer_id and sender != self.peer_id:
              return False
        
        #####################################################################################################################
        ### Update local view
        if node.peer_id not in self.seen_valid_nodes:
          self.seen_valid_nodes[node.peer_id] = []
        self.seen_valid_nodes[node.peer_id].append(node)
        self.pos_in_seen_valid_nodes[node.node_hash] = (node.peer_id, len(self.seen_valid_nodes[node.peer_id]) - 1)

        # store the first inclusion of the transactions at the peer
        if node.peer_id not in self.first_inclusion_of_txs_at_peer:
          self.first_inclusion_of_txs_at_peer[node.peer_id] = {}

        # make a node sees itself so its descendants can use these accumulated values
        node.latest_seen_node_by_peers[node.peer_id] = node.node_hash
        if node.is_witness:
          node.latest_seen_witness_by_peers[node.peer_id] = node.node_hash

        if node.metadata.batch_proposal is not None:
          self.observed_valid_batches[node.metadata.batch_proposal.batch_hash] = node
        for predecessor in self.get_predecessors(node):
          if predecessor.node_hash not in self.local_graph:
            self.local_graph[predecessor.node_hash] = set()
          self.local_graph[predecessor.node_hash].add(node.node_hash)

        # do the cleanup if the node is created by the current peer
        if node.peer_id == self.peer_id:
          self.pending_txs.clear() # because all txs in the pending_txs are now in the new node
          self.current_round = node.round # this makes the current round of the peer = the round of the last node in the list of its nodes
    
        # print(f"Peer {self.peer_id} added node {node.node_hash} to its local view => new round = {self.current_round}")

        for tx in node.newly_seen_txs_list:
          self.first_inclusion_of_txs_at_peer[node.peer_id][tx] = node.node_hash

        # # check consistency
        # for other_peer in self.network.peers:
        #   if other_peer.peer_id == self.peer_id:
        #     continue

        #   print(f"start cons check for {other_peer.peer_id} vs {self.peer_id}")

        #   if other_peer.has_seen_valid_node(node):
        #       l1 = sorted(other_peer.get_node_by_hash(node.node_hash).latest_seen_node_by_peers.values())
        #       l2 = sorted(self.get_node_by_hash(node.node_hash).latest_seen_node_by_peers.values())
        #       if l1 != l2:
        #         ddebug(self.peer_id, f"## CONSISTENCY CHECK FAILED for node {node.node_hash}: {other_peer.peer_id} has {l1} while {self.peer_id} has {l2}")
        #         ddebug(self.peer_id, f"{self.peer_id} => {self.get_node_by_hash(node.node_hash)}, vs {other_peer.peer_id} => {other_peer.get_node_by_hash(node.node_hash)}")
        #         self.network.should_exit = True

        return True

    def find_prev_witness_at_round(self, cur_witness: Node, r: int) -> Optional[Node]:
      while cur_witness.round > r:
        parent_node = self.get_node_by_hash(cur_witness.self_parent_hash)
        if parent_node is None:
          return None

        # jump to prev witness
        prev_witness = self.get_node_by_hash(parent_node.latest_seen_witness_by_peers[cur_witness.peer_id])

        if prev_witness is None:
          return None

        cur_witness = prev_witness

      return cur_witness

    def get_strongly_seen_valid_witnesses(self, dest_node: Node, r: int) -> list["Node"]:
        ## check if this witness strongly sees >= N - f of witnesses of r
        ## if some witnesses are descendants of equivocated nodes, they are ignored completely
        ## NOTE: we already make sure the ancestry of dest_node is verified

        N = len(self.network.peers)

        latest_seen_witness_by_peers: Dict[PeerId, NodeId] = dest_node.latest_seen_witness_by_peers
        seen_witnesses_in_round_gte_r = [self.get_node_by_hash(node_hash) for node_hash in latest_seen_witness_by_peers.values() if node_hash and self.get_node_by_hash(node_hash).round >= r]
        seen_witnesses_in_round_r = [self.find_prev_witness_at_round(node, r) for node in seen_witnesses_in_round_gte_r if node is not None]
        
        for witness in seen_witnesses_in_round_r:
          assert witness.round == r
        
        # keep only the strongly seen ones
        latest_seen_node_by_peers: Dict[PeerId, NodeId] = dest_node.latest_seen_node_by_peers

        seen_nodes_in_round_gte_r = [self.get_node_by_hash(node_hash) for node_hash in latest_seen_node_by_peers.values() if node_hash and self.get_node_by_hash(node_hash).round >= r]
        seen_nodes_in_round_gte_r = [node for node in seen_nodes_in_round_gte_r if node is not None]

        # compute the number of times each witness is seen by each possible (latest) mid node of peers that are seen by the dest_node
        count_seens: Dict[NodeId, int] = {}
        # O(N^2) where N is the number of peers
        for mid_node in seen_nodes_in_round_gte_r:
          if mid_node.peer_id in dest_node.equivocated_peers:
            continue
          for witness in seen_witnesses_in_round_r:
            # we still accept witnesses from equivocated peers but don't count opinions from them
            # check if mid_node can strongly see witness
            witness_peer_id = witness.peer_id

            if witness_peer_id not in mid_node.latest_seen_node_by_peers:
              continue

            latest_seen_node_of_witness_peer_id: NodeId = mid_node.latest_seen_node_by_peers[witness_peer_id]

            if self.is_valid_descendant_and_self_ancestor(latest_seen_node_of_witness_peer_id, witness.node_hash):
              count_seens[witness.node_hash] = count_seens.get(witness.node_hash, 0) + 1
        
        # the dest_node must see the witness of its own peer
        found_self_peer_witness = [witness for witness in seen_witnesses_in_round_r if witness.peer_id == self.peer_id and count_seens.get(witness.node_hash, 0) > 0]
        if len(found_self_peer_witness) == 0:
          return []

        # keep only the witnesses that are seen by N - f of the mid nodes
        strongly_seen_witnesses: list["Node"] = []
        for witness in seen_witnesses_in_round_r:
          if count_seens.get(witness.node_hash, 0) >= self.network.safe_threshold():
            strongly_seen_witnesses.append(witness)
        
        return strongly_seen_witnesses
    
    def check_round_number_of_non_genesis_node_with_valid_parents(self, dest_node: Node) -> bool:
      """
      if a node is of round r:
        - it must not strongly sees >= N - f of witnesses of round r
        - if its self parent is of round r, it is valid. if its self parent is of round r-1, it must strongly sees >= N - f of witnesses of round r-1
      """
      N = len(self.network.peers)
      r = dest_node.round

      self_parent_node = self.get_node_by_hash(dest_node.self_parent_hash)
      (self_parent_node is not None) or (_ for _ in ()).throw(ValueError("self_parent_node must be non-None"))
      if self_parent_node.round < r - 1 or self_parent_node.round > r:
        return False

      if self_parent_node.round == r - 1:
        # the dest_node is a witness of round r so it must strongly sees N - f of witnesses of round r-1
        strongly_seen_witnesses_in_round_r_minus_1 = self.get_strongly_seen_valid_witnesses(dest_node, r-1)
        is_witness_of_round_r = len(strongly_seen_witnesses_in_round_r_minus_1) >= self.network.safe_threshold()

        if not is_witness_of_round_r:
          return False

      # the dest_node must not strongly sees >= N - f of witnesses of round r
      strongly_seen_witnesses_in_round_r = self.get_strongly_seen_valid_witnesses(dest_node, r)

      if len(strongly_seen_witnesses_in_round_r) >= self.network.safe_threshold():
        return False

      return True

    def verify_node(self, node: Node = None) -> bool:
        """Verify a node and its transactions
        - round number must be valid
        - node hash must be valid
        => This method should be called recursively for all ancestors of a node before it's verified

        If the node accepts any parents from an equivocated peer, it is invalid
        """
        if node is None:
          return False

        # if node.peer_id in self.equivocated_peers:
          # return False
        # still receives node from equivocated peers in case the sender is an honest peer

        # an honest peer must not accept a node which itself or its parents are from equivocated peers

        try:
          if node.is_genesis():
            return True
          parent_node = self.get_node_by_hash(node.self_parent_hash)
          # TODO: verify newly_seen_txs_list of the node
          (parent_node is not None) or (_ for _ in ()).throw(ValueError("parent_node must be non-None"))

          valid_node_chain_extension = parent_node.node_hash == self.seen_valid_nodes[parent_node.peer_id][-1].node_hash
          if not valid_node_chain_extension:
            self.equivocated_peers.add(node.peer_id)
            # TODO: log the equivocation activity here
          
          if not node.validate_node_data(self.network.get_peer_pubkey(node.peer_id)):
            return False

          predecessors = self.get_predecessors(node)
          predecessors = self.get_predecessors(node)
          # must have valid parents
          if len(predecessors) < 2:
            return False
          
          predecessors_and_current_node = predecessors + [node]
          for node_to_check in predecessors_and_current_node:
            if node_to_check.peer_id in node.equivocated_peers:
              is_allowed_to_bypass = self.is_adversary and node_to_check.peer_id == self.peer_id # adversary don't accept invalid nodes from other adversaries
              if not is_allowed_to_bypass:
                return False
          # verify the batch proposal if it exists
          
          if node.is_witness:
            is_head_node = self.verify_node_is_head_node(node)
            if is_head_node:
              if not self.verify_batch_proposal_is_valid(node):
                return False
            else:
              (node.metadata.batch_proposal is None) or (_ for _ in ()).throw(ValueError("batch proposal must be None"))
          # verify the vote
          if not self.verify_vote_is_valid(node):
            return False
        except Exception as e:
          # adversary sending invalid nodes
          print("error = ", e)
          return False
        
        is_correct_round_number = self.check_round_number_of_non_genesis_node_with_valid_parents(node)

        return is_correct_round_number

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
        (self_parent is not None and cross_parent is not None) or (_ for _ in ()).throw(ValueError("self_parent and cross_parent must be non-None"))
        if not self.is_adversary:
          (self_parent == self.get_my_last_node()) or (_ for _ in ()).throw(ValueError("self_parent must be the last node of the current peer"))

        (self.get_node_by_hash(self_parent.node_hash) is not None and self.get_node_by_hash(cross_parent.node_hash) is not None) or (_ for _ in ()).throw(ValueError("self_parent and cross_parent must be valid nodes"))

        # the newly seen list of txs in the new node must be not empty
        # TODO: sort this list by timestamp of receipt of the transactions
        newly_seen_txs_list: List[TransactionId] = list(self.calculate_newly_seen_txs_list_of_new_node(self_parent, cross_parent, self.pending_txs))
        
        # if len(newly_seen_txs_list) <= 0:
        #   # can't extend the node sequence because there is no new txs, this is to save the network capacity
        #   return None
        
        round_num = len(self.my_nodes())
        base_hash = f"{self.peer_id}{str(round_num).zfill(3)}"

        if not self.is_adversary:
          (self_parent.round == self.current_round) or (_ for _ in ()).throw(ValueError("self_parent.round must be the current round"))

        print(f"Peer {self.peer_id} start computing new node:")
        for round_num in range(self_parent.round + 1, self_parent.round - 1, -1):
          print(f"Peer {self.peer_id} computing new node for round {round_num}")
          new_node = Node(
              peer_id=self.peer_id,
              height=self_parent.height + 1,
              round=round_num,
              is_witness=False if round_num == self_parent.round else True,
              newly_seen_txs_list=newly_seen_txs_list,
              self_parent_hash=self_parent.node_hash,
              cross_parent_hash=cross_parent.node_hash,
              metadata=NodeMetadata()
          )
          self.fill_node_data(new_node)

          # found a valid new node
          is_node_valid = self.verify_node(new_node)

          if is_node_valid:
            print(f"Peer {self.peer_id} COMPUTED NEW {'HEAD' if new_node.is_head_node() else "NON-HEAD"} NODE {new_node.node_hash} from {self_parent.node_hash} and {cross_parent.node_hash}")
            return new_node
          else:
            print(f"Peer {self.peer_id} computed invalid node {new_node.node_hash} (r={new_node.round}) from {self_parent.node_hash} and {cross_parent.node_hash}")
        # the created new nodes is invalid because either its parents are from equivocated peers
        return None 

    async def gossip_push(self):
        """Sync with another peer through gossip, potentially sending different views"""
        # generate a random permutation of connected peers
        # try to extend the node sequence and push it to the neighbors

        should_equivocate = self.is_adversary and self.network.random_instance.random() < self.equivocation_prob
        self_parent_node = self.get_my_last_node() if not should_equivocate else self.random_instance.choice(self.my_nodes()[-2:])
        # pick a random peer with non-empty seen_valid_nodes
        possible_cross_peers = [peer_id for peer_id in self.seen_valid_nodes if self.seen_valid_nodes[peer_id]]

        if len(possible_cross_peers) <= 1:
          return # can't extend the node sequence because there is no cross parent for the new node

        randomness = min([self.network.random_instance.random(), self.network.random_instance.random()])
        num_nodes_to_create = min(len(possible_cross_peers), 1 + (1 if should_equivocate else 0))
        new_nodes = [] # if there are more than 1 node in this list, they are equivocated nodes and that means current peer is an adversary

        # NOTE: currently, the equivocation logic is simple, an adversary basically picks the last node of the current peer as the self parent, and the latest nodes of different cross peers as the cross parents
        
        for _ in range(num_nodes_to_create):
          max_num_retries = 3

          for i in range(max_num_retries):

            cross_parent_peer_id = self.random_instance.choice(possible_cross_peers)
            for j in range(10):
              if cross_parent_peer_id == self.peer_id or cross_parent_peer_id in self.equivocated_peers:
                cross_parent_peer_id = self.random_instance.choice(possible_cross_peers)
              else:
                break

            if cross_parent_peer_id == self.peer_id or cross_parent_peer_id in self.equivocated_peers or cross_parent_peer_id not in self.seen_valid_nodes or len(self.seen_valid_nodes[cross_parent_peer_id]) <= 0:
              continue

            cross_parent_node = self.seen_valid_nodes[cross_parent_peer_id][-1]

            if self.peer_id == "P7" and self.current_round == 9: # now about to extend to 10
              ddebug(self.peer_id, f"## P7 should_equivocate = {should_equivocate} about to create for round 10: self peer id = {self.peer_id}, self_parent = {self_parent_node.node_hash}, cross peer id = {cross_parent_peer_id}, cross_parent_node = {cross_parent_node.node_hash}")
            
            if cross_parent_node.node_hash in [node.cross_parent_hash for node in new_nodes]:
              # duplicated cross parent
              continue
            
            new_node = self.compute_new_node(self_parent=self_parent_node, cross_parent=cross_parent_node)

            if new_node is not None:
              # found a valid node with unique cross parent
              new_nodes.append(new_node)
              break
            else:
              # Peer can't compute any new nodes from the current tuple of self_parent and cross_parent
              print(f"Peer {self.peer_id} can't compute any new nodes from ({self_parent_node.node_hash}, {cross_parent_node.node_hash})")
              pass

        (len(new_nodes) <= num_nodes_to_create) or (_ for _ in ()).throw(ValueError("number of new nodes must be less than or equal to num_nodes_to_create"))
        if len(new_nodes) <= 0:
          return
        
        for new_node in new_nodes:
          res = self.verify_node_and_add_to_local_view(new_node, sender=self.peer_id)
          if res:
            print(f"Peer {self.peer_id} successfully added self-node {new_node.node_hash} to its local view: {new_node}")
          else:
            print(f"Peer {self.peer_id} failed to add self-node {new_node.node_hash} to its local view: {new_node}")
          (res or self.is_adversary) or (_ for _ in ()).throw(ValueError("failed to verify and add new node"))

        self.select_neighbors(self.network.get_all_peer_ids()) # re-select neighbors

        # start gossiping to neighboring peers
        for i in range(len(self.neighbors)):
            other_peer_id = self.neighbors[i]
            # select randomly nodes from new_nodes
            node_to_send = (new_nodes[0] if i * 2 < len(self.neighbors) else new_nodes[-1]).clone() # simulate the process of serializing and deserializing the nodes in internet protocols

            # Send the selected node
            success = await self.network.gossip_send_node_and_ancestry(self.peer_id, other_peer_id, node_to_send)
            if not success:
              print(f"peer {self.peer_id} gossiped to {other_peer_id} node {node_to_send.node_hash} failed")
            else:
              print(f"peer {self.peer_id} gossiped to {other_peer_id} node {node_to_send.node_hash} successfully")
            
async def main(num_peers, MIN_NUM_ROUNDS_OF_HONEST_PEERS):
    # Create network simulator
    network = NetworkSimulator(
        latency_ms_range=(50, 200),
        packet_loss_prob=0.1,
        random_instance=random.Random(0)
    )

    # Create peers
    count_adversary = 0
    for i in range(num_peers):
        is_adversary = (count_adversary + 1) * 3 + 1 <= num_peers and network.random_instance.random() < 0.5

        peer = ConsensusPeer(
            peer_id=f"P{i}",
            is_adversary=is_adversary,
            seed=i,
            network=network
        )
        if is_adversary:
          count_adversary += 1
          print(f"Peer {peer.peer_id} is an adversary")
        network.register_peer(peer)
    peers = network.peers

    print("Safe threshold = ", network.safe_threshold())

    # Initialize peer neighborhoods
    all_peer_ids = network.get_all_peer_ids()
    for peer in peers:
        peer.select_neighbors(all_peer_ids)

    # Register genesis checkpoint
    await network.register_genesis_nodes()

    current_simluated_timestamp = 0

    time_start = time.time()
    # Main consensus loop
    i = 0
    while not network.should_exit:
        i += 1
        if i % 50 == 0:
          print(f"{i}th iteration at time {time.time() - time_start}, global mempool size = {len(network.global_mempool)}", force=True)
          for peer in peers:
            print(f"Peer {peer.peer_id} (is_adversary={peer.is_adversary}) is at round {peer.current_round} with {len(peer.my_nodes())} nodes", force=True)
        # Count peers that have reached MIN_NUM_ROUNDS_OF_HONEST_PEERS rounds
        peers_completed = sum(1 for c in peers if c.current_round >= MIN_NUM_ROUNDS_OF_HONEST_PEERS)
        if peers_completed >= network.safe_threshold():
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

    for i, peer in enumerate(peers):
      # peer.visualize_view()
      if peer.is_adversary:
        print(f"Peer {peer.peer_id} is an adversary")
      print(f"Neighbors of {peer.peer_id}: {peer.neighbors}")
    print(f"Consensus completed with first {peers_completed} peers reaching round {MIN_NUM_ROUNDS_OF_HONEST_PEERS}")

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

          (found_node_conflict == False) or (_ for _ in ()).throw(ValueError("found node conflict in node info between peers"))
      
      print("SUCCESS: There is no conflict in node info between peers")

    validate_consistency()

    print(f"Total number of transactions = {len(network.global_mempool)}")
    for peer in peers:
      print(f"Peer {peer.peer_id} has {sum([len(peer.seen_valid_nodes[peer_id]) for peer_id in peer.seen_valid_nodes])} nodes")
      print(f"Peer {peer.peer_id} has {len(peer.neighbors)} neighbors")
      print(f"Peer {peer.peer_id} sees that {peer.equivocated_peers} peers are equivocated")
      print(f"Peer {peer.peer_id} is at round {peer.current_round}")

    final_fair_order = None
    for peer in peers:
      peer_final_order = peer.print_final_transaction_order()
      peer_final_order_hash = hashlib.sha256(str(peer_final_order).encode()).hexdigest()
      print(f"Peer {peer.peer_id}'s final txs order: {peer_final_order_hash} : {peer_final_order}, cur round = {peer.current_round}")
      
      if not peer.is_adversary:
        if not final_fair_order: final_fair_order = peer_final_order
        else:
          # find the first index where final_order and final_fair_order differ
          for i in range(min(len(peer_final_order), len(final_fair_order))):
            if peer_final_order[i] != final_fair_order[i]:
              final_fair_order = peer_final_order[:i]
              break

          if len(peer_final_order) < len(final_fair_order):
            final_fair_order = peer_final_order[:]

    print(f"Global mempool size: {len(network.global_mempool)}", force=True)
    print(f"Final fair order: {final_fair_order}", force=True)
    print(f"Ordered txs: {len(final_fair_order)}/{len(network.global_mempool)}={float(len(final_fair_order)) / len(network.global_mempool)} after {MIN_NUM_ROUNDS_OF_HONEST_PEERS} rounds at honest peers", force=True)

time_start = time.time()
asyncio.run(main(num_peers=7, MIN_NUM_ROUNDS_OF_HONEST_PEERS=30))
print(f"Time taken: {time.time() - time_start} seconds")

### Possible attacks:
# Long-Range Attacks: If validators controlling past checkpoints sell their keys, an attacker can re-sign an alternative history, leading to checkpoint reversals.
# => Dangerous once attacker can control N - f of the OLD validators
# Majority Takeover: If an attacker gains control of N - f of the validators (BFT threshold), they could re-finalize a new chain with different checkpoints.
# => recursive validity proof + proof of finality
# Solution: Post-Unstaking Slashing for X blocks after unstaking (but not able to withdraw before X blocks yet)