# Fibonacci Heap in Rust

A high-performance **Fibonacci Heap** implementation in Rust. This data structure is an advanced priority queue that offers improved amortized time complexities compared to binary heaps. It supports efficient operations like **insertion**, **extracting the minimum element**, and **decreasing keys**.

## Features
- **Efficient insertions** with O(1) amortized complexity
- **Quick access** to the minimum element
- **Decrease-key operation** with O(1) amortized complexity
- **Merge two heaps** in O(1) time
- **Extract the minimum element** in O(log n) amortized time

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