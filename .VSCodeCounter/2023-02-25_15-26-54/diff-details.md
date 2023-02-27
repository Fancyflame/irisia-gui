# Diff Details

Date : 2023-02-25 15:26:54

Directory d:\\FancyFlame\\cream-rs

Total : 48 files,  773 codes, 30 comments, 145 blanks, all 948 lines

[Summary](results.md) / [Details](details.md) / [Diff Summary](diff.md) / Diff Details

## Files
| filename | language | code | comment | blank | total |
| :--- | :--- | ---: | ---: | ---: | ---: |
| [cream_core/examples/style_macro.rs](/cream_core/examples/style_macro.rs) | Rust | 72 | 0 | 20 | 92 |
| [cream_core/src/element/mod.rs](/cream_core/src/element/mod.rs) | Rust | -1 | 0 | 0 | -1 |
| [cream_core/src/element/proxy_layer.rs](/cream_core/src/element/proxy_layer.rs) | Rust | -2 | 0 | 0 | -2 |
| [cream_core/src/element/structure/add_child.rs](/cream_core/src/element/structure/add_child.rs) | Rust | -90 | -8 | -11 | -109 |
| [cream_core/src/element/structure/branch.rs](/cream_core/src/element/structure/branch.rs) | Rust | -58 | 0 | -10 | -68 |
| [cream_core/src/element/structure/chain.rs](/cream_core/src/element/structure/chain.rs) | Rust | -32 | 0 | -7 | -39 |
| [cream_core/src/element/structure/mod.rs](/cream_core/src/element/structure/mod.rs) | Rust | -85 | 0 | -15 | -100 |
| [cream_core/src/element/structure/repeating.rs](/cream_core/src/element/structure/repeating.rs) | Rust | -75 | 0 | -13 | -88 |
| [cream_core/src/element/structure/slot.rs](/cream_core/src/element/structure/slot.rs) | Rust | -54 | 0 | -8 | -62 |
| [cream_core/src/lib.rs](/cream_core/src/lib.rs) | Rust | 5 | 0 | 2 | 7 |
| [cream_core/src/macro_helper/chain_caller.rs](/cream_core/src/macro_helper/chain_caller.rs) | Rust | 17 | 0 | 5 | 22 |
| [cream_core/src/macro_helper/for_loop.rs](/cream_core/src/macro_helper/for_loop.rs) | Rust | 20 | 0 | 3 | 23 |
| [cream_core/src/macro_helper/mod.rs](/cream_core/src/macro_helper/mod.rs) | Rust | 4 | 0 | 2 | 6 |
| [cream_core/src/render/mod.rs](/cream_core/src/render/mod.rs) | Rust | 2 | 0 | 0 | 2 |
| [cream_core/src/structure/add_child.rs](/cream_core/src/structure/add_child.rs) | Rust | 101 | 8 | 12 | 121 |
| [cream_core/src/structure/branch.rs](/cream_core/src/structure/branch.rs) | Rust | 58 | 0 | 10 | 68 |
| [cream_core/src/structure/chain.rs](/cream_core/src/structure/chain.rs) | Rust | 32 | 0 | 7 | 39 |
| [cream_core/src/structure/mod.rs](/cream_core/src/structure/mod.rs) | Rust | 73 | 6 | 17 | 96 |
| [cream_core/src/structure/repeating.rs](/cream_core/src/structure/repeating.rs) | Rust | 85 | 0 | 14 | 99 |
| [cream_core/src/structure/slot.rs](/cream_core/src/structure/slot.rs) | Rust | 62 | 0 | 9 | 71 |
| [cream_core/src/style/add_style.rs](/cream_core/src/style/add_style.rs) | Rust | 19 | 0 | 5 | 24 |
| [cream_core/src/style/chain.rs](/cream_core/src/style/chain.rs) | Rust | 24 | 0 | 4 | 28 |
| [cream_core/src/style/mod.rs](/cream_core/src/style/mod.rs) | Rust | -5 | 0 | 1 | -4 |
| [cream_core/src/style/utils.rs](/cream_core/src/style/utils.rs) | Rust | -32 | 0 | -6 | -38 |
| [cream_macros/Cargo.toml](/cream_macros/Cargo.toml) | TOML | 2 | 0 | 0 | 2 |
| [cream_macros/src/elem_mod.rs](/cream_macros/src/elem_mod.rs) | Rust | -19 | 0 | -3 | -22 |
| [cream_macros/src/element/cmd.rs](/cream_macros/src/element/cmd.rs) | Rust | 15 | 0 | 3 | 18 |
| [cream_macros/src/element/mod.rs](/cream_macros/src/element/mod.rs) | Rust | 45 | 0 | 12 | 57 |
| [cream_macros/src/element/stmt.rs](/cream_macros/src/element/stmt.rs) | Rust | 15 | 0 | 4 | 19 |
| [cream_macros/src/expr/conditional/ca.rs](/cream_macros/src/expr/conditional/ca.rs) | Rust | 50 | 0 | 11 | 61 |
| [cream_macros/src/expr/conditional/mod.rs](/cream_macros/src/expr/conditional/mod.rs) | Rust | 3 | 0 | 1 | 4 |
| [cream_macros/src/expr/conditional/state_if.rs](/cream_macros/src/expr/conditional/state_if.rs) | Rust | 81 | 12 | 15 | 108 |
| [cream_macros/src/expr/conditional/state_match.rs](/cream_macros/src/expr/conditional/state_match.rs) | Rust | 98 | 7 | 14 | 119 |
| [cream_macros/src/expr/mod.rs](/cream_macros/src/expr/mod.rs) | Rust | 81 | 0 | 15 | 96 |
| [cream_macros/src/expr/repetitive/mod.rs](/cream_macros/src/expr/repetitive/mod.rs) | Rust | 2 | 0 | 1 | 3 |
| [cream_macros/src/expr/repetitive/state_for.rs](/cream_macros/src/expr/repetitive/state_for.rs) | Rust | 49 | 3 | 7 | 59 |
| [cream_macros/src/expr/repetitive/state_while.rs](/cream_macros/src/expr/repetitive/state_while.rs) | Rust | 48 | 0 | 7 | 55 |
| [cream_macros/src/expr/state_block.rs](/cream_macros/src/expr/state_block.rs) | Rust | 72 | 1 | 9 | 82 |
| [cream_macros/src/expr/state_command.rs](/cream_macros/src/expr/state_command.rs) | Rust | 68 | 0 | 9 | 77 |
| [cream_macros/src/lib.rs](/cream_macros/src/lib.rs) | Rust | 2 | 1 | -2 | 1 |
| [cream_macros/src/proc_control/conditional/mod.rs](/cream_macros/src/proc_control/conditional/mod.rs) | Rust | -12 | 0 | -4 | -16 |
| [cream_macros/src/proc_control/conditional/state_if.rs](/cream_macros/src/proc_control/conditional/state_if.rs) | Rust | -36 | 0 | -8 | -44 |
| [cream_macros/src/proc_control/conditional/state_match.rs](/cream_macros/src/proc_control/conditional/state_match.rs) | Rust | -52 | 0 | -7 | -59 |
| [cream_macros/src/proc_control/mod.rs](/cream_macros/src/proc_control/mod.rs) | Rust | -27 | 0 | -6 | -33 |
| [cream_macros/src/proc_control/repetitive/mod.rs](/cream_macros/src/proc_control/repetitive/mod.rs) | Rust | 0 | 0 | -2 | -2 |
| [cream_macros/src/proc_control/state_block.rs](/cream_macros/src/proc_control/state_block.rs) | Rust | -28 | 0 | -4 | -32 |
| [cream_macros/src/style/mod.rs](/cream_macros/src/style/mod.rs) | Rust | 45 | 0 | 14 | 59 |
| [cream_macros/src/style/stmt.rs](/cream_macros/src/style/stmt.rs) | Rust | 131 | 0 | 18 | 149 |

[Summary](results.md) / [Details](details.md) / [Diff Summary](diff.md) / Diff Details