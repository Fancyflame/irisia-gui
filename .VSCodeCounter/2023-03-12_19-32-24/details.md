# Details

Date : 2023-03-12 19:32:24

Directory d:\\FancyFlame\\cream-rs

Total : 87 files,  3728 codes, 74 comments, 757 blanks, all 4559 lines

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)

## Files
| filename | language | code | comment | blank | total |
| :--- | :--- | ---: | ---: | ---: | ---: |
| [Cargo.toml](/Cargo.toml) | TOML | 3 | 0 | 1 | 4 |
| [README.md](/README.md) | Markdown | 3 | 0 | 2 | 5 |
| [cream/Cargo.toml](/cream/Cargo.toml) | TOML | 7 | 1 | 3 | 11 |
| [cream/src/event.rs](/cream/src/event.rs) | Rust | 10 | 0 | 3 | 13 |
| [cream/src/lib.rs](/cream/src/lib.rs) | Rust | 1 | 0 | 1 | 2 |
| [cream_backend/Cargo.toml](/cream_backend/Cargo.toml) | TOML | 17 | 1 | 5 | 23 |
| [cream_backend/src/application.rs](/cream_backend/src/application.rs) | Rust | 8 | 0 | 3 | 11 |
| [cream_backend/src/lib.rs](/cream_backend/src/lib.rs) | Rust | 6 | 0 | 3 | 9 |
| [cream_backend/src/runtime/global.rs](/cream_backend/src/runtime/global.rs) | Rust | 23 | 0 | 8 | 31 |
| [cream_backend/src/runtime/mod.rs](/cream_backend/src/runtime/mod.rs) | Rust | 35 | 0 | 9 | 44 |
| [cream_backend/src/runtime/rt_event.rs](/cream_backend/src/runtime/rt_event.rs) | Rust | 20 | 0 | 5 | 25 |
| [cream_backend/src/runtime/send_event.rs](/cream_backend/src/runtime/send_event.rs) | Rust | 53 | 0 | 11 | 64 |
| [cream_backend/src/window/create.rs](/cream_backend/src/window/create.rs) | Rust | 28 | 0 | 6 | 34 |
| [cream_backend/src/window/mod.rs](/cream_backend/src/window/mod.rs) | Rust | 62 | 0 | 12 | 74 |
| [cream_backend/src/window/renderer.rs](/cream_backend/src/window/renderer.rs) | Rust | 76 | 0 | 20 | 96 |
| [cream_backend/src/window/run.rs](/cream_backend/src/window/run.rs) | Rust | 57 | 0 | 14 | 71 |
| [cream_backend/src/window/timer.rs](/cream_backend/src/window/timer.rs) | Rust | 119 | 0 | 20 | 139 |
| [cream_core/Cargo.toml](/cream_core/Cargo.toml) | TOML | 9 | 1 | 3 | 13 |
| [cream_core/examples/style_macro.rs](/cream_core/examples/style_macro.rs) | Rust | 72 | 0 | 20 | 92 |
| [cream_core/examples/test.rs](/cream_core/examples/test.rs) | Rust | 74 | 0 | 15 | 89 |
| [cream_core/src/element/mod.rs](/cream_core/src/element/mod.rs) | Rust | 34 | 5 | 11 | 50 |
| [cream_core/src/element/proxy_layer.rs](/cream_core/src/element/proxy_layer.rs) | Rust | 57 | 0 | 4 | 61 |
| [cream_core/src/element/render_content.rs](/cream_core/src/element/render_content.rs) | Rust | 38 | 0 | 9 | 47 |
| [cream_core/src/event/event_flow.rs](/cream_core/src/event/event_flow.rs) | Rust | 40 | 0 | 11 | 51 |
| [cream_core/src/event/event_state/build/add_listener.rs](/cream_core/src/event/event_state/build/add_listener.rs) | Rust | 41 | 0 | 7 | 48 |
| [cream_core/src/event/event_state/build/chain.rs](/cream_core/src/event/event_state/build/chain.rs) | Rust | 27 | 0 | 7 | 34 |
| [cream_core/src/event/event_state/build/empty.rs](/cream_core/src/event/event_state/build/empty.rs) | Rust | 18 | 0 | 5 | 23 |
| [cream_core/src/event/event_state/build/mod.rs](/cream_core/src/event/event_state/build/mod.rs) | Rust | 58 | 0 | 10 | 68 |
| [cream_core/src/event/event_state/mod.rs](/cream_core/src/event/event_state/mod.rs) | Rust | 15 | 0 | 4 | 19 |
| [cream_core/src/event/event_state/proxy/add_listener2.rs](/cream_core/src/event/event_state/proxy/add_listener2.rs) | Rust | 41 | 0 | 7 | 48 |
| [cream_core/src/event/event_state/proxy/mod.rs](/cream_core/src/event/event_state/proxy/mod.rs) | Rust | 48 | 0 | 9 | 57 |
| [cream_core/src/event/event_state/proxy/proxy.rs](/cream_core/src/event/event_state/proxy/proxy.rs) | Rust | 103 | 1 | 15 | 119 |
| [cream_core/src/event/event_state/wrap/inner.rs](/cream_core/src/event/event_state/wrap/inner.rs) | Rust | 41 | 0 | 9 | 50 |
| [cream_core/src/event/event_state/wrap/mod.rs](/cream_core/src/event/event_state/wrap/mod.rs) | Rust | 43 | 0 | 11 | 54 |
| [cream_core/src/event/global_register/mod.rs](/cream_core/src/event/global_register/mod.rs) | Rust | 6 | 0 | 4 | 10 |
| [cream_core/src/event/global_register/system_event_register.rs](/cream_core/src/event/global_register/system_event_register.rs) | Rust | 70 | 0 | 18 | 88 |
| [cream_core/src/event/mod.rs](/cream_core/src/event/mod.rs) | Rust | 6 | 0 | 4 | 10 |
| [cream_core/src/lib.rs](/cream_core/src/lib.rs) | Rust | 14 | 1 | 6 | 21 |
| [cream_core/src/macro_helper/chain_caller.rs](/cream_core/src/macro_helper/chain_caller.rs) | Rust | 17 | 0 | 5 | 22 |
| [cream_core/src/macro_helper/for_loop.rs](/cream_core/src/macro_helper/for_loop.rs) | Rust | 20 | 0 | 3 | 23 |
| [cream_core/src/macro_helper/mod.rs](/cream_core/src/macro_helper/mod.rs) | Rust | 5 | 1 | 3 | 9 |
| [cream_core/src/primary.rs](/cream_core/src/primary.rs) | Rust | 40 | 2 | 9 | 51 |
| [cream_core/src/render/mod.rs](/cream_core/src/render/mod.rs) | Rust | 54 | 0 | 8 | 62 |
| [cream_core/src/structure/add_child/mod.rs](/cream_core/src/structure/add_child/mod.rs) | Rust | 102 | 8 | 15 | 125 |
| [cream_core/src/structure/add_child/pl_cache.rs](/cream_core/src/structure/add_child/pl_cache.rs) | Rust | 17 | 0 | 3 | 20 |
| [cream_core/src/structure/branch.rs](/cream_core/src/structure/branch.rs) | Rust | 57 | 0 | 10 | 67 |
| [cream_core/src/structure/cache_box.rs](/cream_core/src/structure/cache_box.rs) | Rust | 35 | 0 | 5 | 40 |
| [cream_core/src/structure/chain.rs](/cream_core/src/structure/chain.rs) | Rust | 29 | 0 | 7 | 36 |
| [cream_core/src/structure/fn_helper.rs](/cream_core/src/structure/fn_helper.rs) | Rust | 15 | 0 | 3 | 18 |
| [cream_core/src/structure/mod.rs](/cream_core/src/structure/mod.rs) | Rust | 59 | 5 | 16 | 80 |
| [cream_core/src/structure/repeating.rs](/cream_core/src/structure/repeating.rs) | Rust | 82 | 0 | 14 | 96 |
| [cream_core/src/structure/slot.rs](/cream_core/src/structure/slot.rs) | Rust | 60 | 0 | 9 | 69 |
| [cream_core/src/style/add_style.rs](/cream_core/src/style/add_style.rs) | Rust | 19 | 0 | 5 | 24 |
| [cream_core/src/style/branch.rs](/cream_core/src/style/branch.rs) | Rust | 18 | 0 | 3 | 21 |
| [cream_core/src/style/chain.rs](/cream_core/src/style/chain.rs) | Rust | 24 | 0 | 4 | 28 |
| [cream_core/src/style/mod.rs](/cream_core/src/style/mod.rs) | Rust | 23 | 25 | 9 | 57 |
| [cream_core/src/style/reader.rs](/cream_core/src/style/reader.rs) | Rust | 49 | 0 | 6 | 55 |
| [cream_macros/Cargo.toml](/cream_macros/Cargo.toml) | TOML | 11 | 1 | 4 | 16 |
| [cream_macros/src/element/build.rs](/cream_macros/src/element/build.rs) | Rust | 94 | 0 | 16 | 110 |
| [cream_macros/src/element/cmd.rs](/cream_macros/src/element/cmd.rs) | Rust | 24 | 0 | 3 | 27 |
| [cream_macros/src/element/mod.rs](/cream_macros/src/element/mod.rs) | Rust | 67 | 0 | 14 | 81 |
| [cream_macros/src/element/stmt/mod.rs](/cream_macros/src/element/stmt/mod.rs) | Rust | 23 | 0 | 7 | 30 |
| [cream_macros/src/element/stmt/parse.rs](/cream_macros/src/element/stmt/parse.rs) | Rust | 99 | 1 | 17 | 117 |
| [cream_macros/src/element/stmt/to_tokens.rs](/cream_macros/src/element/stmt/to_tokens.rs) | Rust | 64 | 0 | 12 | 76 |
| [cream_macros/src/expr/conditional/ca.rs](/cream_macros/src/expr/conditional/ca.rs) | Rust | 50 | 0 | 11 | 61 |
| [cream_macros/src/expr/conditional/mod.rs](/cream_macros/src/expr/conditional/mod.rs) | Rust | 3 | 0 | 1 | 4 |
| [cream_macros/src/expr/conditional/state_if.rs](/cream_macros/src/expr/conditional/state_if.rs) | Rust | 107 | 12 | 21 | 140 |
| [cream_macros/src/expr/conditional/state_match.rs](/cream_macros/src/expr/conditional/state_match.rs) | Rust | 118 | 7 | 16 | 141 |
| [cream_macros/src/expr/mod.rs](/cream_macros/src/expr/mod.rs) | Rust | 50 | 1 | 14 | 65 |
| [cream_macros/src/expr/repetitive/mod.rs](/cream_macros/src/expr/repetitive/mod.rs) | Rust | 2 | 0 | 1 | 3 |
| [cream_macros/src/expr/repetitive/state_for.rs](/cream_macros/src/expr/repetitive/state_for.rs) | Rust | 63 | 0 | 10 | 73 |
| [cream_macros/src/expr/repetitive/state_while.rs](/cream_macros/src/expr/repetitive/state_while.rs) | Rust | 62 | 0 | 9 | 71 |
| [cream_macros/src/expr/state_block.rs](/cream_macros/src/expr/state_block.rs) | Rust | 97 | 1 | 11 | 109 |
| [cream_macros/src/expr/state_command.rs](/cream_macros/src/expr/state_command.rs) | Rust | 52 | 0 | 9 | 61 |
| [cream_macros/src/expr/state_expr.rs](/cream_macros/src/expr/state_expr.rs) | Rust | 93 | 0 | 10 | 103 |
| [cream_macros/src/lib.rs](/cream_macros/src/lib.rs) | Rust | 37 | 0 | 6 | 43 |
| [cream_macros/src/style/mod.rs](/cream_macros/src/style/mod.rs) | Rust | 45 | 0 | 14 | 59 |
| [cream_macros/src/style/stmt.rs](/cream_macros/src/style/stmt.rs) | Rust | 131 | 0 | 18 | 149 |
| [cream_macros/src/t.rs](/cream_macros/src/t.rs) | Rust | 26 | 0 | 4 | 30 |
| [documentation/comp_mngr/summary.md](/documentation/comp_mngr/summary.md) | Markdown | 22 | 0 | 7 | 29 |
| [documentation/element/computed.md](/documentation/element/computed.md) | Markdown | 83 | 0 | 20 | 103 |
| [documentation/element/responsible_data.md](/documentation/element/responsible_data.md) | Markdown | 25 | 0 | 7 | 32 |
| [documentation/element/style.md](/documentation/element/style.md) | Markdown | 47 | 0 | 14 | 61 |
| [documentation/element/summary.md](/documentation/element/summary.md) | Markdown | 26 | 0 | 10 | 36 |
| [documentation/index.md](/documentation/index.md) | Markdown | 85 | 0 | 16 | 101 |
| [documentation/renderer.md](/documentation/renderer.md) | Markdown | 13 | 0 | 3 | 16 |
| [documentation/summary.svg](/documentation/summary.svg) | XML | 1 | 0 | 0 | 1 |

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)