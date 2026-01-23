
# Examples

This is a tiny collection of bobcat-sdk examples, including some tests to spot codesize
regressions between releases. Examples that mirror the reference versions must remain
functionally identical. These examples (except Camelot) don't need the allocator, but
they're brought in to make compilation possible from the workspace due to feature
unification.

For a more complete illustration of what the SDK offers, check the end-to-end tests. Build
each example from its own project directory.

## Codesize comparison (in bytes)

|   Name   | stylus-sdk (0.9.2) | bobcat-sdk |                          Description                           |
|----------|--------------------|------------|----------------------------------------------------------------|
| Counter  | 16807              | 5592       | A simple counter app that does basic manipulation of storage.  |
| Muldiv   | 19332              | 4337       | A muldiv implementation, compared to the version in 9lives.    |
| Camelot  | 45360              | 4527       | Acts as an intermediary for Camelot swapping using its router. |
| Bozo     | N/A                | 28494      | The contract code powering (https://ripbozo.lol)[Ripbozo].     |
