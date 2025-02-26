//! Fibonacci Heap implementation in Rust
//!
//! This module provides a Fibonacci Heap, a priority queue with better
//! amortized time complexity compared to a binary heap.
//!
//! # Features
//! - Efficient insertions and merging
//! - Quick access to the minimum element
//! - Support for decrease-key operations
//!
//! # Example Usage
//!
//! ## Linking Nodes (During Consolidation)
//! ```rust
//! use fibonacci_heap::FibonacciHeap;
//!
//! let mut heap = FibonacciHeap::new();
//! let n1 = heap.insert(10);
//! let n2 = heap.insert(20);
//! let n3 = heap.insert(5);
//!
//! // Extract min triggers consolidation
//! heap.extract_min();
//!
//! // Nodes 10 and 20 get linked during consolidation
//! assert_eq!(heap.root_list.len(), 1);
//! ```
//!
//! ## Cutting Nodes (During Decrease-Key)
//! ```rust
//! use fibonacci_heap::FibonacciHeap;
//!
//! let mut heap = FibonacciHeap::new();
//! heap.insert(30);
//! let node = heap.insert(40);
//! heap.insert(10);
//! heap.extract_min(); // Forces consolidation
//!
//! // Decrease key triggers cut from parent
//! heap.decrease_key(&node, 5);
//!
//! // Node 40 (now 5) is promoted to root
//! assert_eq!(heap.extract_min(), Some(5));
//! ```
//!
//! ## Cascading Cut (Multi-Level Cutting)
//! ```rust
//! use fibonacci_heap::FibonacciHeap;
//!
//! let mut heap = FibonacciHeap::new();
//! heap.insert(100);
//! let parent = heap.insert(200);
//! heap.insert(50);
//! heap.extract_min(); // Consolidate to create hierarchy
//!
//! heap.decrease_key(&parent, 150);
//! let child = heap.insert(300);
//! heap.extract_min(); // Create parent-child-grandchild
//!
//! // Trigger cascading cut
//! heap.decrease_key(&child, 10); // First cut
//! heap.decrease_key(&parent, 20); // Second cut triggers cascade
//!
//! assert_eq!(heap.extract_min(), Some(10));
//! assert_eq!(heap.extract_min(), Some(20));
//! ```
//!
//! ## Full Workflow with All Operations
//! ```rust
//! use fibonacci_heap::FibonacciHeap;
//!
//! let mut heap = FibonacciHeap::new();
//! let node_30 = heap.insert(30);
//! heap.insert(10);
//! let node_20 = heap.insert(20);
//! assert_eq!(heap.extract_min(), Some(10));
//!
//! let node_40 = heap.insert(40);
//! heap.insert(5);
//! assert_eq!(heap.extract_min(), Some(5));
//!
//! // Decrease keys using stored node references
//! heap.decrease_key(&node_40, 2);
//! heap.decrease_key(&node_30, 1);
//!
//! assert_eq!(heap.extract_min(), Some(1));  // From node_30
//! assert_eq!(heap.extract_min(), Some(2));  // From node_40
//! assert_eq!(heap.extract_min(), Some(20)); // Original node_20
//! ```
//!
//! # Complexity
//! | Operation     | Amortized Complexity |
//! |--------------|----------------------|
//! | Insert       | O(1)                 |
//! | Merge        | O(1)                 |
//! | Extract Min  | O(log n)             |
//! | Decrease Key | O(1)                 |
//!

use std::sync::{Arc, Mutex};

type NodePtr = Arc<Mutex<Node>>;

/// Represents a node in the Fibonacci Heap.
#[derive(Clone)]
pub struct Node {
    pub key: i32,                  // The key value of the node
    degree: usize,                 // Number of children
    marked: bool,                  // Whether the node has lost a child
    parent: Option<NodePtr>,       // Reference to the parent node
    children: Vec<NodePtr>,        // List of child nodes
}

impl Node {
    /// Creates a new node with the given key.
    pub fn new(key: i32) -> NodePtr {
        Arc::new(Mutex::new(Node {
            key,
            degree: 0,
            marked: false,
            parent: None,
            children: Vec::new(),
        }))
    }
}

/// Represents a Fibonacci Heap data structure.
pub struct FibonacciHeap {
    min: Option<NodePtr>,          // The minimum node
    pub root_list: Vec<NodePtr>,   // List of roots
}

impl Default for FibonacciHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl FibonacciHeap {
    /// Creates a new, empty Fibonacci Heap.
    pub fn new() -> Self {
        FibonacciHeap {
            min: None,
            root_list: Vec::new(),
        }
    }

    /// Inserts a new key into the heap and returns the created node.
    pub fn insert(&mut self, key: i32) -> NodePtr {
        let node = Node::new(key);
        self.root_list.push(node.clone());

        if let Some(ref min) = self.min {
            if key < min.lock().unwrap().key {
                self.min = Some(node.clone());
            }
        } else {
            self.min = Some(node.clone());
        }

        node
    }

    /// Merges another Fibonacci Heap into this one.
    pub fn merge(&mut self, other: &mut FibonacciHeap) {
        self.root_list.append(&mut other.root_list);

        if let Some(ref other_min) = other.min {
            match &self.min {
                Some(self_min) if other_min.lock().unwrap().key < self_min.lock().unwrap().key => {
                    self.min = Some(other_min.clone());
                }
                None => {
                    self.min = Some(other_min.clone());
                }
                _ => {}
            }
        }

        other.root_list.clear();
        other.min = None;
    }

    /// Extracts the minimum key from the heap and restructures it.
    pub fn extract_min(&mut self) -> Option<i32> {
        let min_node = self.min.take()?;

        let min_key = min_node.lock().unwrap().key;
        let children = std::mem::take(&mut min_node.lock().unwrap().children);
        for child in &children {
            child.lock().unwrap().parent = None;
            self.root_list.push(child.clone());
        }

        self.root_list.retain(|node| !Arc::ptr_eq(node, &min_node));

        if self.root_list.is_empty() {
            self.min = None;
        } else {
            self.consolidate();
        }

        Some(min_key)
    }

    /// Consolidates the heap by linking nodes of the same degree.
    fn consolidate(&mut self) {
        const MAX_DEGREE: usize = 64;
        let mut degrees: [Option<NodePtr>; MAX_DEGREE] = std::array::from_fn(|_| None);
        let original_root_list = std::mem::take(&mut self.root_list);
        let mut new_min = None;

        for node in original_root_list {
            let mut current = node;
            let mut degree = current.lock().unwrap().degree;

            while let Some(existing) = degrees[degree].take() {
                let current_key = current.lock().unwrap().key;
                let existing_key = existing.lock().unwrap().key;
                if existing_key < current_key {
                    self.link(current.clone(), existing.clone());
                    current = existing;
                } else {
                    self.link(existing.clone(), current.clone());
                }
                degree = current.lock().unwrap().degree;
                if degree >= MAX_DEGREE {
                    panic!("Degree exceeded MAX_DEGREE!");
                }
            }
            degrees[degree] = Some(current.clone());

            let current_key = current.lock().unwrap().key;
            new_min = match new_min {
                None => Some(current.clone()),
                Some(min) if current_key < min.lock().unwrap().key => Some(current.clone()),
                min => min,
            };
        }

        self.root_list = degrees.into_iter().flatten().collect();
        self.min = new_min;
    }

    /// Links one node as a child of another.
    fn link(&mut self, node: NodePtr, parent: NodePtr) {
        {
            let mut node_guard = node.lock().unwrap();
            node_guard.parent = Some(parent.clone());
            node_guard.marked = false;
        }
        {
            let mut parent_guard = parent.lock().unwrap();
            parent_guard.children.push(node.clone());
            parent_guard.degree += 1;
        }
    }

    /// Decreases the key of a node and performs necessary heap operations.
    pub fn decrease_key(&mut self, node: &NodePtr, new_key: i32) {
        let mut node_guard = node.lock().unwrap();
        if new_key > node_guard.key {
            panic!("New key is greater than current key!");
        }

        node_guard.key = new_key;
        let parent = node_guard.parent.clone();
        drop(node_guard);

        if let Some(parent_ref) = parent {
            if new_key < parent_ref.lock().unwrap().key {
                self.cut(node.clone(), parent_ref.clone());
                self.cascading_cut(parent_ref);
            }
        }

        if let Some(ref current_min) = self.min {
            if new_key < current_min.lock().unwrap().key {
                self.min = Some(node.clone());
            }
        } else {
            self.min = Some(node.clone());
        }
    }

    /// Cuts a node from its parent and moves it to the root list.
    fn cut(&mut self, node: NodePtr, parent: NodePtr) {
        let mut parent_guard = parent.lock().unwrap();
        parent_guard.children.retain(|child| !Arc::ptr_eq(child, &node));
        parent_guard.degree -= 1;
        drop(parent_guard);

        let mut node_guard = node.lock().unwrap();
        self.root_list.push(node.clone());
        node_guard.parent = None;
        node_guard.marked = false;
    }

    /// Performs a cascading cut operation on a node's ancestors.
    fn cascading_cut(&mut self, node: NodePtr) {
        let parent = node.lock().unwrap().parent.clone();
        if let Some(parent_ref) = parent {
            let marked = node.lock().unwrap().marked;
            if !marked {
                node.lock().unwrap().marked = true;
            } else {
                self.cut(node.clone(), parent_ref.clone());
                self.cascading_cut(parent_ref);
            }
        }
    }

    /// Checks if the heap is empty.
    pub fn is_empty(&self) -> bool {
        self.root_list.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut heap = FibonacciHeap::new();
        let node = heap.insert(10);
        assert_eq!(node.lock().unwrap().key, 10);
        assert_eq!(heap.min.unwrap().lock().unwrap().key, 10);
    }

    #[test]
    fn test_extract_min() {
        let mut heap = FibonacciHeap::new();
        heap.insert(10);
        heap.insert(20);
        heap.insert(5);
        let min = heap.extract_min();
        assert_eq!(min, Some(5));
        assert_eq!(heap.min.unwrap().lock().unwrap().key, 10);
    }

    #[test]
    fn test_merge() {
        let mut heap1 = FibonacciHeap::new();
        let mut heap2 = FibonacciHeap::new();
        heap1.insert(10);
        heap2.insert(5);
        heap2.insert(20);
        heap1.merge(&mut heap2);
        assert_eq!(heap1.extract_min(), Some(5));
    }

    #[test]
    fn test_decrease_key() {
        let mut heap = FibonacciHeap::new();
        let node = heap.insert(10);
        heap.decrease_key(&node, 5);
        assert_eq!(heap.min.unwrap().lock().unwrap().key, 5);
    }

    #[test]
    #[should_panic(expected = "New key is greater than current key!")]
    fn test_decrease_key_invalid() {
        let mut heap = FibonacciHeap::new();
        let node = heap.insert(10);
        heap.decrease_key(&node, 15);
    }

    #[test]
    fn test_empty_heap() {
        let heap = FibonacciHeap::new();
        assert!(heap.is_empty());
    }
}