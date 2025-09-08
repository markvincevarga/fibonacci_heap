//! A high-performance Fibonacci Heap implementation in Rust.
//!
//! Fibonacci Heap is a collection of trees that satisfies the minimum heap property.
//! It provides efficient operations for insertion, merging, and decreasing keys,
//! making it ideal for algorithms like Dijkstra's and Prim's.
//!
//! # Features
//! - O(1) amortized time for insert and merge operations
//! - O(1) amortized time for decrease key operations
//! - O(log n) amortized time for extract minimum operations
//! - Comprehensive error handling
//! - Works with any type implementing `Ord + Clone`
//!
//! # Example
//! ```
//! use fibonacci_heap::FibonacciHeap;
//!
//! let mut heap = FibonacciHeap::new();
//! let node1 = heap.insert(10).unwrap();
//! let node2 = heap.insert(5).unwrap();
//! assert_eq!(heap.extract_min(), Some(5));
//!
//! heap.decrease_key(&node1, 3).unwrap();
//! assert_eq!(heap.extract_min(), Some(3));
//! ```

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

/// Error types for Fibonacci Heap operations
#[derive(Debug, PartialEq)]
pub enum HeapError {
    InvalidKey,
    NodeNotFound,
    HeapEmpty,
}

/// A node in the Fibonacci Heap
#[derive(Debug)]
pub struct Node<T> {
    pub key: T,
    degree: usize,
    marked: bool,
    parent: Option<Weak<RefCell<Node<T>>>>,
    children: Vec<Rc<RefCell<Node<T>>>>,
    id: usize, // Unique identifier for node validation
}

impl<T> Node<T> {
    /// Creates a new node with the given key and unique ID
    fn new(key: T, id: usize) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            key,
            degree: 0,
            marked: false,
            parent: None,
            children: Vec::new(),
            id,
        }))
    }
}

/// A Fibonacci Heap data structure
#[derive(Debug)]
pub struct FibonacciHeap<T> {
    min: Option<Rc<RefCell<Node<T>>>>,
    root_list: Vec<Rc<RefCell<Node<T>>>>,
    node_count: usize,
    next_id: AtomicUsize,
    active_nodes: HashMap<usize, Weak<RefCell<Node<T>>>>,
}

impl<T: Ord + Clone> Default for FibonacciHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + Clone> FibonacciHeap<T> {
    /// Creates a new empty Fibonacci Heap
    ///
    /// # Examples
    /// ```
    /// use fibonacci_heap::FibonacciHeap;
    /// let heap = FibonacciHeap::<i32>::new();
    /// assert!(heap.is_empty());
    /// ```
    pub fn new() -> Self {
        FibonacciHeap {
            min: None,
            root_list: Vec::new(),
            node_count: 0,
            next_id: AtomicUsize::new(0),
            active_nodes: HashMap::new(),
        }
    }

    /// Inserts a new key into the heap and returns a reference to the created node
    ///
    /// # Arguments
    /// * `key` - The value to insert
    ///
    /// # Returns
    /// `Result` containing a node reference or an error
    ///
    /// # Examples
    /// ```
    /// use fibonacci_heap::FibonacciHeap;
    /// let mut heap = FibonacciHeap::new();
    /// let node = heap.insert(42).unwrap();
    /// ```
    pub fn insert(&mut self, key: T) -> Result<Rc<RefCell<Node<T>>>, HeapError> {
        let id = self.next_id.fetch_add(1, AtomicOrdering::SeqCst);
        let node = Node::new(key, id);

        // Store weak reference for validation
        self.active_nodes.insert(id, Rc::downgrade(&node));

        self.root_list.push(Rc::clone(&node));
        self.node_count += 1;

        // Update minimum if needed
        match &self.min {
            Some(min) if node.borrow().key < min.borrow().key => {
                self.min = Some(Rc::clone(&node));
            }
            None => self.min = Some(Rc::clone(&node)),
            _ => (),
        }

        Ok(node)
    }

    /// Merges another Fibonacci Heap into this one
    ///
    /// # Arguments
    /// * `other` - The heap to merge into this one
    ///
    /// # Examples
    /// ```
    /// use fibonacci_heap::FibonacciHeap;
    ///
    /// let mut heap1 = FibonacciHeap::new();
    /// heap1.insert(10).unwrap();
    ///
    /// let mut heap2 = FibonacciHeap::new();
    /// heap2.insert(5).unwrap();
    ///
    /// heap1.merge(heap2);
    /// assert_eq!(heap1.extract_min(), Some(5));
    /// ```
    pub fn merge(&mut self, other: FibonacciHeap<T>) {
        // Merge root lists
        self.root_list.extend(other.root_list);
        self.node_count += other.node_count;

        // Merge active nodes
        self.active_nodes.extend(other.active_nodes);

        // Update minimum if needed
        if let Some(other_min) = other.min {
            match &self.min {
                Some(self_min) if other_min.borrow().key < self_min.borrow().key => {
                    self.min = Some(other_min);
                }
                None => self.min = Some(other_min),
                _ => (),
            }
        }
    }

    /// Extracts the minimum value from the heap
    ///
    /// # Returns
    /// The minimum value or `None` if the heap is empty
    ///
    /// # Examples
    /// ```
    /// use fibonacci_heap::FibonacciHeap;
    ///
    /// let mut heap = FibonacciHeap::new();
    /// heap.insert(10).unwrap();
    /// heap.insert(5).unwrap();
    ///
    /// assert_eq!(heap.extract_min(), Some(5));
    /// ```
    pub fn extract_min(&mut self) -> Option<T> {
        let min_node = self.min.take()?;
        let min_key = min_node.borrow().key.clone();
        let min_id = min_node.borrow().id;

        // Remove from active nodes
        self.active_nodes.remove(&min_id);

        // Add children to root list
        let children = std::mem::take(&mut min_node.borrow_mut().children);
        for child in children {
            child.borrow_mut().parent = None;
            self.root_list.push(child);
        }

        // Remove min node from root list
        self.root_list.retain(|node| !Rc::ptr_eq(node, &min_node));
        self.node_count -= 1;

        if self.root_list.is_empty() {
            self.min = None;
        } else {
            self.consolidate();
        }

        Some(min_key)
    }

    /// Consolidates the trees in the heap to maintain the Fibonacci Heap properties
    fn consolidate(&mut self) {
        // Calculate maximum possible degree based on node count
        let max_degree = (self.node_count as f64).log2() as usize + 2;
        let mut degree_table: Vec<Option<Rc<RefCell<Node<T>>>>> = vec![None; max_degree];
        let mut new_min = None;

        // Process all root nodes
        let roots = std::mem::take(&mut self.root_list);
        for root in roots {
            let mut current = root;
            let mut degree = current.borrow().degree;

            // Combine trees with same degree
            while let Some(existing) = degree_table[degree].take() {
                if current.borrow().key < existing.borrow().key {
                    self.link(existing, &current);
                } else {
                    self.link(current, &existing);
                    current = existing;
                }
                degree = current.borrow().degree;

                // Extend degree table if needed
                if degree >= degree_table.len() {
                    degree_table.resize(degree + 1, None);
                }
            }

            degree_table[degree] = Some(current.clone());

            // Track new minimum
            if new_min
                .as_ref()
                .is_none_or(|min: &Rc<RefCell<Node<T>>>| current.borrow().key < min.borrow().key)
            {
                new_min = Some(current);
            }
        }

        // Rebuild root list from degree table
        self.root_list = degree_table.into_iter().flatten().collect();
        self.min = new_min;
    }

    /// Links two trees by making one a child of the other
    fn link(&mut self, child: Rc<RefCell<Node<T>>>, parent: &Rc<RefCell<Node<T>>>) {
        // Remove child from root list
        self.root_list.retain(|node| !Rc::ptr_eq(node, &child));

        // Update child's parent
        child.borrow_mut().parent = Some(Rc::downgrade(parent));
        child.borrow_mut().marked = false;

        // Add child to parent's children
        parent.borrow_mut().children.push(child);
        parent.borrow_mut().degree += 1;
    }

    /// Decreases the key of a node
    ///
    /// # Arguments
    /// * `node` - Reference to the node to update
    /// * `new_key` - The new key value
    ///
    /// # Returns
    /// `Result` indicating success or an error
    ///
    /// # Examples
    /// ```
    /// use fibonacci_heap::FibonacciHeap;
    ///
    /// let mut heap = FibonacciHeap::new();
    /// let node = heap.insert(20).unwrap();
    /// heap.insert(10).unwrap();
    ///
    /// assert_eq!(heap.extract_min(), Some(10));
    /// heap.decrease_key(&node, 5).unwrap();
    /// assert_eq!(heap.extract_min(), Some(5));
    /// ```
    pub fn decrease_key(
        &mut self,
        node: &Rc<RefCell<Node<T>>>,
        new_key: T,
    ) -> Result<(), HeapError> {
        // Validate node reference
        let node_id = node.borrow().id;
        if !self.active_nodes.contains_key(&node_id) {
            return Err(HeapError::NodeNotFound);
        }

        // Validate key
        if new_key > node.borrow().key {
            return Err(HeapError::InvalidKey);
        }

        // Update key
        node.borrow_mut().key = new_key.clone();

        // Check if heap property is violated
        if let Some(parent_weak) = &node.borrow().parent {
            if let Some(parent) = parent_weak.upgrade() {
                if new_key < parent.borrow().key {
                    self.cut(node, &parent);
                    self.cascading_cut(&parent);
                }
            }
        }

        // Update minimum if needed
        if self.min.is_none() || new_key < self.min.as_ref().unwrap().borrow().key {
            self.min = Some(Rc::clone(node));
        }

        Ok(())
    }

    /// Cuts a node from its parent and moves it to the root list
    fn cut(&mut self, node: &Rc<RefCell<Node<T>>>, parent: &Rc<RefCell<Node<T>>>) {
        // Remove node from parent's children
        parent
            .borrow_mut()
            .children
            .retain(|child| !Rc::ptr_eq(child, node));
        parent.borrow_mut().degree -= 1;

        // Add node to root list
        node.borrow_mut().parent = None;
        node.borrow_mut().marked = false;
        self.root_list.push(Rc::clone(node));
    }

    /// Performs cascading cuts on a node's ancestors if needed
    fn cascading_cut(&mut self, node: &Rc<RefCell<Node<T>>>) {
        if let Some(parent_weak) = &node.borrow().parent {
            if let Some(parent) = parent_weak.upgrade() {
                if !node.borrow().marked {
                    node.borrow_mut().marked = true;
                } else {
                    self.cut(node, &parent);
                    self.cascading_cut(&parent);
                }
            }
        }
    }

    /// Returns the minimum value without removing it
    ///
    /// # Returns
    /// The minimum value or `None` if the heap is empty
    ///
    /// # Examples
    /// ```
    /// use fibonacci_heap::FibonacciHeap;
    ///
    /// let mut heap = FibonacciHeap::new();
    /// heap.insert(10).unwrap();
    /// heap.insert(5).unwrap();
    ///
    /// assert_eq!(heap.peek_min(), Some(5));
    /// ```
    pub fn peek_min(&self) -> Option<T> {
        self.min.as_ref().map(|min| min.borrow().key.clone())
    }

    /// Returns a cloned copy of the minimum value without removing it
    pub fn peek_min_cloned(&self) -> Option<T> {
        self.min.as_ref().map(|min| min.borrow().key.clone())
    }

    /// Checks if the heap is empty
    ///
    /// # Returns
    /// `true` if the heap is empty, `false` otherwise
    ///
    /// # Examples
    /// ```
    /// use fibonacci_heap::FibonacciHeap;
    ///
    /// let heap = FibonacciHeap::new();
    /// assert!(heap.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.root_list.is_empty()
    }

    /// Returns the number of nodes in the heap
    ///
    /// # Returns
    /// The number of nodes in the heap
    ///
    /// # Examples
    /// ```
    /// use fibonacci_heap::FibonacciHeap;
    ///
    /// let mut heap = FibonacciHeap::new();
    /// heap.insert(10).unwrap();
    /// heap.insert(20).unwrap();
    ///
    /// assert_eq!(heap.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.node_count
    }

    /// Clears the heap, removing all values
    ///
    /// # Examples
    /// ```
    /// use fibonacci_heap::FibonacciHeap;
    ///
    /// let mut heap = FibonacciHeap::new();
    /// heap.insert(10).unwrap();
    /// heap.clear();
    ///
    /// assert!(heap.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.min = None;
        self.root_list.clear();
        self.node_count = 0;
        self.active_nodes.clear();
        self.next_id.store(0, AtomicOrdering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Task {
        priority: i32,
        name: String,
    }

    impl Ord for Task {
        fn cmp(&self, other: &Self) -> Ordering {
            self.priority.cmp(&other.priority)
        }
    }

    impl PartialOrd for Task {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    #[test]
    fn test_basic_operations_i32() {
        let mut heap = FibonacciHeap::new();
        assert!(heap.is_empty());

        heap.insert(10).unwrap();
        heap.insert(5).unwrap();
        assert_eq!(heap.len(), 2);

        assert_eq!(heap.extract_min(), Some(5));
        assert_eq!(heap.extract_min(), Some(10));
        assert!(heap.is_empty());
    }

    #[test]
    fn test_basic_operations_string() {
        let mut heap = FibonacciHeap::new();

        heap.insert("zebra".to_string()).unwrap();
        heap.insert("apple".to_string()).unwrap();
        heap.insert("banana".to_string()).unwrap();

        assert_eq!(heap.extract_min(), Some("apple".to_string()));
        assert_eq!(heap.extract_min(), Some("banana".to_string()));
        assert_eq!(heap.extract_min(), Some("zebra".to_string()));
    }

    #[test]
    fn test_custom_type() {
        let mut heap = FibonacciHeap::new();

        let task1 = Task {
            priority: 10,
            name: "Low priority".to_string(),
        };
        let task2 = Task {
            priority: 1,
            name: "High priority".to_string(),
        };
        let task3 = Task {
            priority: 5,
            name: "Medium priority".to_string(),
        };

        heap.insert(task1).unwrap();
        heap.insert(task2.clone()).unwrap();
        heap.insert(task3).unwrap();

        assert_eq!(heap.extract_min().unwrap().name, "High priority");
        assert_eq!(heap.extract_min().unwrap().name, "Medium priority");
        assert_eq!(heap.extract_min().unwrap().name, "Low priority");
    }

    #[test]
    fn test_merge_generic() {
        let mut heap1 = FibonacciHeap::new();
        heap1.insert(10).unwrap();
        heap1.insert(20).unwrap();

        let mut heap2 = FibonacciHeap::new();
        heap2.insert(5).unwrap();
        heap2.insert(15).unwrap();

        heap1.merge(heap2);
        assert_eq!(heap1.len(), 4);
        assert_eq!(heap1.extract_min(), Some(5));
    }

    #[test]
    fn test_decrease_key_generic() {
        let mut heap = FibonacciHeap::new();
        let node = heap.insert(20).unwrap();
        heap.insert(10).unwrap();

        assert_eq!(heap.extract_min(), Some(10));
        heap.decrease_key(&node, 5).unwrap();
        assert_eq!(heap.extract_min(), Some(5));
    }

    #[test]
    fn test_decrease_key_validation_generic() {
        let mut heap = FibonacciHeap::new();
        let node = heap.insert(10).unwrap();

        // Invalid key
        assert_eq!(heap.decrease_key(&node, 15), Err(HeapError::InvalidKey));

        // Valid key
        assert!(heap.decrease_key(&node, 5).is_ok());
    }

    #[test]
    fn test_peek_operations() {
        let mut heap = FibonacciHeap::new();
        heap.insert(10).unwrap();
        heap.insert(5).unwrap();
        heap.insert(15).unwrap();

        assert_eq!(heap.peek_min(), Some(5));
        assert_eq!(heap.peek_min_cloned(), Some(5));
        assert_eq!(heap.len(), 3); // Peek shouldn't remove items
    }

    #[test]
    fn test_decrease_key_custom_type() {
        let mut heap = FibonacciHeap::new();

        let high_task = Task {
            priority: 10,
            name: "Initial low priority".to_string(),
        };
        let node = heap.insert(high_task).unwrap();
        heap.insert(Task {
            priority: 5,
            name: "Medium priority".to_string(),
        })
        .unwrap();

        let updated_task = Task {
            priority: 1,
            name: "Now high priority".to_string(),
        };
        heap.decrease_key(&node, updated_task.clone()).unwrap();

        assert_eq!(heap.extract_min().unwrap().name, "Now high priority");
    }
}
