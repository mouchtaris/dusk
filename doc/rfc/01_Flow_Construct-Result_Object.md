---
direction: Proposal
affects: vm, syntax
---
# Introducing control flow with the Result-type Object

[2.1] This proposal describes the idea of using Result-typed Objects, 
[2.2] which are the `Result<>` type with it's api exposed at runtime.

## Background

[1.1] `dust` is designed to be a reliable pipeline tool.
[1.2] It has exactly zero tolerance for any drift from an expected path of execution.
[1.3] No process can exit with non-zero exit-code once it has existed.
[1.4] This is a feature 99% of the time, but allows for no "effectful" management.
[1.5] According to theory, any effectful management for "serious", top-level, unix-inhabiting application would have to be delegated to the appropriate external unix tool.

## Problem Statement

[3.1] However, in cases, we hit the convolution spiral the other way around, and we need to contain effectful computations as primitives. 
[3.2] {Example} A good example of this is running a test suit:
[3.3] we need to know the exit status as a data-point, not as an inline event in our horizon.
[3.4] In such situations, using a full blow unix failure management tool is contra-principles.
[3.5] Therefor, in such a case, a lightweight, purpose-craft binary or sys-call primitive should be used.

## Next Steps:

[4.1] Discuss next steps.
[4.2] Discuss approaches.

## Blocking Items:

[5.1] Document is a draft.

## ToDo Items:
[6.1] - [ ] Give first syntax to enable discussion

## Scrut process
This is the scrutinization process of this document:
1. Frontmatter `affects`: validate accuracy
2. No Blocking Items
