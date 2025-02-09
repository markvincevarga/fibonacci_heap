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
//! ```rust
//! use fibonacci_heap::FibonacciHeap;
//!
//! let mut heap = FibonacciHeap::new();
//! heap.insert(10);
//! heap.insert(20);
//! let min = heap.extract_min();
//! assert_eq!(min, Some(10));
//! ```
//!
//! # Complexity
//! | Operation     | Amortized Complexity |
//! |--------------|--------------------|
//! | Insert       | O(1)               |
//! | Merge        | O(1)               |
//! | Extract Min  | O(log n)           |
//! | Decrease Key | O(1)               |
//!

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Represents a node in the Fibonacci Heap.
#[derive(PartialEq, Clone)]
pub struct Node {
    pub key: i32,                      // The key value of the node
    degree: usize,                     // Number of children
    marked: bool,                      // Whether the node has lost a child
    parent: Option<Rc<RefCell<Node>>>, // Reference to the parent node
    children: Vec<Rc<RefCell<Node>>>,  // List of child nodes
}

impl Node {
    /// Creates a new node with the given key.
    pub fn new(key: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
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
    min: Option<Rc<RefCell<Node>>>,            // The minimum node
    root_list: Vec<Rc<RefCell<Node>>>,         // List of roots
    node_map: HashMap<i32, Rc<RefCell<Node>>>, // Map of keys to nodes
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
            node_map: HashMap::new(),
        }
    }

    /// Inserts a new key into the heap and returns the created node.
    pub fn insert(&mut self, key: i32) -> Rc<RefCell<Node>> {
        let node = Node::new(key);
        self.root_list.push(node.clone());
        self.node_map.insert(key, node.clone());

        // Update the minimum node if necessary
        if let Some(ref mut min) = self.min {
            if key < min.borrow().key {
                self.min = Some(node.clone());
            }
        } else {
            self.min = Some(node.clone());
        }

        node
    }

    /// Merges another Fibonacci Heap into this one.
    pub fn merge(&mut self, other: &mut FibonacciHeap) {
        if let Some(ref min_node) = other.min {
            self.root_list.push(min_node.clone());
            if let Some(ref mut min) = self.min {
                if min_node.borrow().key < min.borrow().key {
                    self.min = Some(min_node.clone());
                }
            } else {
                self.min = Some(min_node.clone());
            }
        }
        self.root_list.append(&mut other.root_list);
        self.node_map.extend(other.node_map.drain());
        other.root_list.clear();
        other.node_map.clear();
    }

    /// Extracts the minimum key from the heap and restructures it.
    pub fn extract_min(&mut self) -> Option<i32> {
        let min_node = self.min.take();

        if let Some(min_node_ref) = min_node {
            let min_key = min_node_ref.borrow().key;

            // Move children to root list before removing the node
            let children = std::mem::take(&mut min_node_ref.borrow_mut().children);
            for child in &children {
                child.borrow_mut().parent = None;
                self.root_list.push(child.clone());
            }

            // Remove min_node from root list
            self.root_list
                .retain(|node| !Rc::ptr_eq(node, &min_node_ref));

            if self.root_list.is_empty() {
                self.min = None;
            } else {
                self.consolidate();
            }

            Some(min_key)
        } else {
            None
        }
    }

    /// Consolidates the heap by linking nodes of the same degree.
    fn consolidate(&mut self) {
        let mut degrees: Vec<Option<Rc<RefCell<Node>>>> = Vec::new();
        let original_root_list = std::mem::take(&mut self.root_list);

        for node in original_root_list {
            let mut current = node;
            let mut degree = current.borrow().degree;

            loop {
                // Ensure the degrees vector is large enough to accommodate the current degree
                while degree >= degrees.len() {
                    degrees.push(None);
                }

                // Check if there's a tree with the same degree
                if let Some(existing_node) = degrees[degree].take() {
                    if existing_node.borrow().key < current.borrow().key {
                        // Existing node has a smaller key, so it becomes the parent
                        self.link(current.clone(), existing_node.clone());
                        current = existing_node;
                    } else {
                        // Current node has a smaller key, existing node becomes its child
                        self.link(existing_node.clone(), current.clone());
                    }
                    // The degree of the current node has increased (due to linking)
                    degree = current.borrow().degree;
                } else {
                    // No existing node with this degree, insert current into degrees
                    degrees[degree] = Some(current.clone());
                    break;
                }
            }
        }

        // Rebuild the root list from the degrees vector
        self.root_list = degrees.into_iter().flatten().collect();

        // Find the new minimum node
        self.min = self.root_list.iter().min_by_key(|node| node.borrow().key).cloned();
    }

    /// Links one node as a child of another.
    fn link(&mut self, node: Rc<RefCell<Node>>, parent: Rc<RefCell<Node>>) {
        node.borrow_mut().parent = Some(parent.clone());
        parent.borrow_mut().children.push(node.clone());
        parent.borrow_mut().degree += 1;
        node.borrow_mut().marked = false;
    }

    /// Decreases the key of a node and performs necessary heap operations.
    pub fn decrease_key(&mut self, node_key: i32, new_key: i32) {
        let node = match self.node_map.get(&node_key) {
            Some(rc) => rc.clone(),
            None => return,
        };

        {
            let node_borrowed = node.borrow();
            if new_key > node_borrowed.key {
                panic!("New key is greater than current key!");
            }
        }

        let parent = {
            let mut node_borrowed = node.borrow_mut();
            node_borrowed.key = new_key;
            node_borrowed.parent.clone()
        };

        if let Some(parent_ref) = parent {
            if new_key < parent_ref.borrow().key {
                self.cut(node.clone(), parent_ref.clone());
                self.cascading_cut(parent_ref);
            }
        }

        self.node_map.remove(&node_key);
        self.node_map.insert(new_key, node.clone());

        if let Some(ref current_min) = self.min {
            if new_key < current_min.borrow().key {
                self.min = Some(node);
            }
        } else {
            self.min = Some(node);
        }
    }

    /// Cuts a node from its parent and moves it to the root list.
    fn cut(&mut self, node: Rc<RefCell<Node>>, parent: Rc<RefCell<Node>>) {
        let mut parent_borrowed = parent.borrow_mut();
        parent_borrowed
            .children
            .retain(|child| !Rc::ptr_eq(child, &node));
        parent_borrowed.degree -= 1;
        self.root_list.push(node.clone());
        node.borrow_mut().parent = None;
        node.borrow_mut().marked = false;
    }

    /// Performs a cascading cut operation on a node's ancestors.
    fn cascading_cut(&mut self, node: Rc<RefCell<Node>>) {
        if let Some(parent) = node.borrow().parent.clone() {
            if !node.borrow().marked {
                node.borrow_mut().marked = true;
            } else {
                self.cut(node.clone(), parent.clone());
                self.cascading_cut(parent);
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
        assert_eq!(node.borrow().key, 10);
        assert_eq!(heap.min.unwrap().borrow().key, 10);
    }

    #[test]
    fn test_extract_min() {
        let mut heap = FibonacciHeap::new();
        heap.insert(10);
        heap.insert(20);
        heap.insert(5);
        let min = heap.extract_min();
        assert_eq!(min, Some(5));
        assert_eq!(heap.min.unwrap().borrow().key, 10);
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
        let _node = heap.insert(10);
        heap.decrease_key(10, 5);
        assert_eq!(heap.min.unwrap().borrow().key, 5);
    }

    #[test]
    #[should_panic(expected = "New key is greater than current key!")]
    fn test_decrease_key_invalid() {
        let mut heap = FibonacciHeap::new();
        let _node = heap.insert(10);
        heap.decrease_key(10, 15);
    }

    #[test]
    fn test_empty_heap() {
        let heap = FibonacciHeap::new();
        assert!(heap.is_empty());
    }

    #[test]
    fn test_multiple_extracts() {
        let mut heap = FibonacciHeap::new();
        heap.insert(10);
        heap.insert(5);
        heap.insert(20);
        println!(
            "Inserted 20, root list: {:?}",
            heap.root_list
                .iter()
                .map(|n| n.borrow().key)
                .collect::<Vec<_>>()
        );
        assert_eq!(heap.extract_min(), Some(5));
        assert_eq!(heap.extract_min(), Some(10));
        assert_eq!(heap.extract_min(), Some(20));
        assert_eq!(heap.extract_min(), None);
    }
}
