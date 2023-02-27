# Details

Date : 2023-02-25 15:27:49

Directory d:\\FancyFlame\\cream-rs

Total : 68 files,  2578 codes, 74 comments, 537 blanks, all 3189 lines

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)

## Files
| filename | language | code | comment | blank | total |
| :--- | :--- | ---: | ---: | ---: | ---: |
| [Cargo.toml](/Cargo.toml) | TOML | 2 | 0 | 1 | 3 |
| [README.md](/README.md) | Markdown | 3 | 0 | 2 | 5 |
| [cream_core/Cargo.toml](/cream_core/Cargo.toml) | TOML | 9 | 1 | 3 | 13 |
| [cream_core/examples/style_macro.rs](/cream_core/examples/style_macro.rs) | Rust | 72 | 0 | 20 | 92 |
| [cream_core/src/element/mod.rs](/cream_core/src/element/mod.rs) | Rust | 32 | 5 | 10 | 47 |
| [cream_core/src/element/proxy_layer.rs](/cream_core/src/element/proxy_layer.rs) | Rust | 48 | 0 | 7 | 55 |
| [cream_core/src/element/render_content.rs](/cream_core/src/element/render_content.rs) | Rust | 35 | 0 | 8 | 43 |
| [cream_core/src/event/event_flow.rs](/cream_core/src/event/event_flow.rs) | Rust | 40 | 0 | 11 | 51 |
| [cream_core/src/event/global_register/mod.rs](/cream_core/src/event/global_register/mod.rs) | Rust | 6 | 0 | 4 | 10 |
| [cream_core/src/event/global_register/system_event_register.rs](/cream_core/src/event/global_register/system_event_register.rs) | Rust | 70 | 0 | 18 | 88 |
| [cream_core/src/event/listeners/add_listener.rs](/cream_core/src/event/listeners/add_listener.rs) | Rust | 28 | 0 | 7 | 35 |
| [cream_core/src/event/listeners/chain.rs](/cream_core/src/event/listeners/chain.rs) | Rust | 17 | 0 | 6 | 23 |
| [cream_core/src/event/listeners/mod.rs](/cream_core/src/event/listeners/mod.rs) | Rust | 38 | 0 | 13 | 51 |
| [cream_core/src/event/listeners/owned/mod.rs](/cream_core/src/event/listeners/owned/mod.rs) | Rust | 64 | 0 | 15 | 79 |
| [cream_core/src/event/listeners/owned/output.rs](/cream_core/src/event/listeners/owned/output.rs) | Rust | 32 | 0 | 8 | 40 |
| [cream_core/src/event/mod.rs](/cream_core/src/event/mod.rs) | Rust | 5 | 0 | 3 | 8 |
| [cream_core/src/lib.rs](/cream_core/src/lib.rs) | Rust | 14 | 0 | 5 | 19 |
| [cream_core/src/macro_helper/chain_caller.rs](/cream_core/src/macro_helper/chain_caller.rs) | Rust | 17 | 0 | 5 | 22 |
| [cream_core/src/macro_helper/for_loop.rs](/cream_core/src/macro_helper/for_loop.rs) | Rust | 20 | 0 | 3 | 23 |
| [cream_core/src/macro_helper/mod.rs](/cream_core/src/macro_helper/mod.rs) | Rust | 4 | 0 | 2 | 6 |
| [cream_core/src/primary.rs](/cream_core/src/primary.rs) | Rust | 40 | 2 | 9 | 51 |
| [cream_core/src/render/mod.rs](/cream_core/src/render/mod.rs) | Rust | 85 | 0 | 11 | 96 |
| [cream_core/src/structure/add_child.rs](/cream_core/src/structure/add_child.rs) | Rust | 101 | 8 | 12 | 121 |
| [cream_core/src/structure/branch.rs](/cream_core/src/structure/branch.rs) | Rust | 58 | 0 | 10 | 68 |
| [cream_core/src/structure/chain.rs](/cream_core/src/structure/chain.rs) | Rust | 32 | 0 | 7 | 39 |
| [cream_core/src/structure/mod.rs](/cream_core/src/structure/mod.rs) | Rust | 73 | 6 | 17 | 96 |
| [cream_core/src/structure/repeating.rs](/cream_core/src/structure/repeating.rs) | Rust | 85 | 0 | 14 | 99 |
| [cream_core/src/structure/slot.rs](/cream_core/src/structure/slot.rs) | Rust | 62 | 0 | 9 | 71 |
| [cream_core/src/style/add_style.rs](/cream_core/src/style/add_style.rs) | Rust | 19 | 0 | 5 | 24 |
| [cream_core/src/style/branch.rs](/cream_core/src/style/branch.rs) | Rust | 18 | 0 | 3 | 21 |
| [cream_core/src/style/chain.rs](/cream_core/src/style/chain.rs) | Rust | 24 | 0 | 4 | 28 |
| [cream_core/src/style/mod.rs](/cream_core/src/style/mod.rs) | Rust | 23 | 25 | 8 | 56 |
| [cream_core/src/style/reader.rs](/cream_core/src/style/reader.rs) | Rust | 49 | 0 | 6 | 55 |
| [cream_core/src/style/units.rs](/cream_core/src/style/units.rs) | Rust | 4 | 0 | 2 | 6 |
| [cream_kit/Cargo.toml](/cream_kit/Cargo.toml) | TOML | 6 | 1 | 3 | 10 |
| [cream_kit/src/event.rs](/cream_kit/src/event.rs) | Rust | 10 | 0 | 3 | 13 |
| [cream_kit/src/lib.rs](/cream_kit/src/lib.rs) | Rust | 1 | 0 | 1 | 2 |
| [cream_macros/Cargo.toml](/cream_macros/Cargo.toml) | TOML | 11 | 1 | 4 | 16 |
| [cream_macros/examples/t.rs](/cream_macros/examples/t.rs) | Rust | 11 | 0 | 2 | 13 |
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
| [cream_macros/src/lib.rs](/cream_macros/src/lib.rs) | Rust | 39 | 1 | 6 | 46 |
| [cream_macros/src/style/mod.rs](/cream_macros/src/style/mod.rs) | Rust | 45 | 0 | 14 | 59 |
| [cream_macros/src/style/stmt.rs](/cream_macros/src/style/stmt.rs) | Rust | 131 | 0 | 18 | 149 |
| [cream_macros/src/t.rs](/cream_macros/src/t.rs) | Rust | 26 | 0 | 4 | 30 |
| [cream_macros/src/uninit_type/mod.rs](/cream_macros/src/uninit_type/mod.rs) | Rust | 63 | 0 | 12 | 75 |
| [cream_macros/src/uninit_type/set.rs](/cream_macros/src/uninit_type/set.rs) | Rust | 60 | 0 | 11 | 71 |
| [cream_winit/Cargo.toml](/cream_winit/Cargo.toml) | TOML | 5 | 1 | 3 | 9 |
| [cream_winit/src/lib.rs](/cream_winit/src/lib.rs) | Rust | 12 | 0 | 3 | 15 |
| [documentation/comp_mngr/summary.md](/documentation/comp_mngr/summary.md) | Markdown | 22 | 0 | 7 | 29 |
| [documentation/element/computed.md](/documentation/element/computed.md) | Markdown | 83 | 0 | 20 | 103 |
| [documentation/element/responsible_data.md](/documentation/element/responsible_data.md) | Markdown | 25 | 0 | 7 | 32 |
| [documentation/element/style.md](/documentation/element/style.md) | Markdown | 47 | 0 | 14 | 61 |
| [documentation/element/summary.md](/documentation/element/summary.md) | Markdown | 26 | 0 | 10 | 36 |
| [documentation/index.md](/documentation/index.md) | Markdown | 85 | 0 | 16 | 101 |
| [documentation/renderer.md](/documentation/renderer.md) | Markdown | 13 | 0 | 3 | 16 |
| [documentation/summary.svg](/documentation/summary.svg) | XML | 1 | 0 | 0 | 1 |

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)