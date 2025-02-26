[![Crates.io](https://img.shields.io/crates/v/fibonacci_heap.svg)](https://crates.io/crates/fibonacci_heap)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)
![License](https://img.shields.io/badge/License-MIT-blue)

# Fibonacci Heap in Rust


A high-performance **Fibonacci Heap** implementation in Rust. The **Fibonacci Heap** is a heap data structure consisting of a collection of trees, which is used to implement priority queues. It offers improved amortized time complexities for many operations compared to other heap structures, such as binary heaps. Its primary advantages are its efficient **decrease-key** and **merge** operations, making it particularly useful in algorithms like Dijkstra's and Prim's for shortest paths and minimum spanning trees.


## Features

### Time Complexities
- **Insertions:** O(1) amortized complexity
- **Extract Minimum:** O(log n) amortized complexity
- **Decrease Key:** O(1) amortized complexity
- **Merge two heaps:** O(1) time complexity

### Supported Operations
- **Insert:** Add a new element to the heap.
- **Extract Min:** Remove the element with the smallest value.
- **Decrease Key:** Modify the value of an element, reducing it.

### Internal Operations

- **Link:** The `link` operation is used to link two trees in the heap when the root of one tree becomes smaller than the root of another tree. It connects the smaller tree as a child of the larger tree. This operation helps maintain the heap property and is a key part of the Fibonacci heap structure, contributing to the efficient decrease-key and merge operations.

- **Cut:** The `cut` operation removes a node from its parent in the heap, and places it as a new root. This operation is used when the decrease-key operation causes a node's value to become smaller than its parent's value, violating the heap property. Cutting the node ensures the heap structure remains valid and allows for efficient future operations.

- **Cascading Cut:** The `cascading_cut` operation is a recursive process that cuts a node and propagates the cut up the tree. If a node’s parent has already lost a child (i.e., been cut before), the node itself is recursively cut and moved to the root list. This process helps maintain a balanced structure in the Fibonacci heap, ensuring that each node’s degree is not too large, which contributes to the heap’s efficient performance.


## Implementation Details

- The heap is represented as a collection of trees.
- Each tree is a root of a doubly linked list, where nodes are linked to their parent and siblings.
- The decrease-key operation is efficient due to the lazy structure of the heap.


## Example Usage

Here's an example of how to use the Fibonacci Heap in your project:

```rust
use fibonacci_heap::FibonacciHeap;

fn main() {
    let mut heap = FibonacciHeap::new();

    // Insert elements
    heap.insert(10);
    heap.insert(20);

    // Extract the minimum element
    let min = heap.extract_min();
    println!("Extracted min: {:?}", min);  // Output: Some(10)

    // Decrease key
    let node = heap.insert(30);
    heap.decrease_key(30, 5);
    let min_after_decrease = heap.extract_min();
    println!("Extracted min after decrease key: {:?}", min_after_decrease);  // Output: Some(5)
}
```

## Example Tests
- test_insert: Inserts a single element and checks if the heap's minimum is correct.
- test_extract_min: Inserts elements and ensures the correct minimum is extracted.
- test_merge: Merges two Fibonacci heaps and checks the minimum after the merge.
- test_decrease_key: Tests the decrease key operation.
- test_multiple_extracts: Extracts multiple elements and checks the order.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

# Contributing

Contributions are welcome! Please fork this repository, make your changes, and submit a pull request. Ensure that all tests pass before submitting.

# Contact

If you have any questions or suggestions, feel free to open an issue or reach out to the author:

Author: xvi-xv-xii-ix-xxii-ix-xiv
GitHub Repository: https://github.com/xvi-xv-xii-ix-xxii-ix-xiv/fibonacci_heap
Documentation: https://docs.rs/fibonacci_heap