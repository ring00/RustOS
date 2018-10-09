# ucore-memory
Memory management in RustOS is composed of three parts:
* **Platform dependent details**: e.g. `CR3`. In `kernel/src/arch/xxx`.
* **Platform independent parts**: this crate.
* **Gluing them together**: in `kernel/src/memory.rs`.

This crate contains platform independent parts of RustOS memory management,
including
* Transparent wrapper to an address space offering fork-COW functionality.
* Transparent wrapper to an address space offering swapping to disk.
* Some testing utils.
