## 1. Search Focus Resource and Clear Button Infrastructure

- [x] 1.1 Add `SearchFocused(pub bool)` resource to `toolbar.rs` (derive `Resource`, `Debug`, `Clone`, `Default`), and `init_resource::<SearchFocused>()` in `ExplorerViewPlugin::build`
- [x] 1.2 Add `SearchClearButton` marker component to `toolbar.rs`
- [x] 1.3 In `spawn_toolbar_into`, make `SearchInputField` a `Button` node so it receives `Interaction` events; add a `SearchClearButton` child inside it (label `"×"`, 14 px, `colors::GREY1`; initial `display: Display::None`)
- [x] 1.4 Add system `handle_search_field_click` in `toolbar.rs`: when `SearchInputField` button `Interaction::Pressed`, set `SearchFocused(true)`; add `#[tracing::instrument(skip_all)]`
- [x] 1.5 Add system `update_search_field_visuals` in `toolbar.rs`: toggle `SearchClearButton` node `display` (`Display::Flex` when `SearchQuery` non-empty, else `Display::None`); update `SearchInputField` border color to `ORANGE` when `SearchFocused`, else `GREY1`; guard with `if !search.is_changed() && !focused.is_changed() { return; }`
- [x] 1.6 Add system `handle_search_clear_click` in `toolbar.rs`: when `SearchClearButton` `Interaction::Pressed`, reset `SearchQuery("")` and `SearchFocused(false)`; add `#[tracing::instrument(skip_all)]`

## 2. Keyboard Input Handler

- [x] 2.1 Replace the stub body of `handle_text_search_input` in `toolbar.rs` with real logic: guard `if edit_mode.editing_index.is_some() || !focused.0 { keyboard.clear(); return; }`; branch on `KeyCode`: `Escape` → `SearchFocused(false)` (keep query), `Backspace` → `search.0.pop()`, other → append `event.text` chars that are not control characters; add `edit_mode: Res<CardEditMode>` and `focused: ResMut<SearchFocused>` to the system params
- [x] 2.2 In `handle_card_inline_edit` (`card.rs`), add a guard at the top: `if search_focused.0 { keyboard.clear(); return; }` and add `search_focused: Res<toolbar::SearchFocused>` to the system params

## 3. Fuzzy Match and `CardMatchHighlight` Component

- [x] 3.1 Add `CardMatchHighlight` component to `card.rs`: `pub struct CardMatchHighlight { pub name_range: Option<(usize, usize)>, pub id_range: Option<(usize, usize)> }` (derive `Component`, `Debug`, `Clone`, `Default`)
- [x] 3.2 Add a free function `fuzzy_match(query: &str, target: &str) -> Option<(usize, usize)>` in `toolbar.rs` (or a shared `search.rs` module): character-subsequence walk over `target.char_indices()`, returns byte offsets `(start, end)` of the matched span (first char match start to last char match end), returns `None` if any query char is absent; case-insensitive (both lowercased before comparison)
- [x] 3.3 In `apply_filter_and_sort` (`toolbar.rs`), replace the substring `contains` check with `fuzzy_match`; after visibility is determined, insert/mutate `CardMatchHighlight` on each card entity: set `name_range` from fuzzy match on display name string, `id_range` from fuzzy match on `get_id()`; set both to `None` when `query_lower` is empty; requires adding `mut commands: Commands` and `CardMatchHighlight` to the query

## 4. Highlight Rendering on Card Labels

- [x] 4.1 Restructure `CardNameLabel` and `CardIdLabel` spawning in `spawn_card` (`card.rs`) to use Bevy 0.16 multi-span text: spawn the label root with `Text::default()` and three `TextSpan` children tagged `LabelSpanPrefix`, `LabelSpanHighlight`, `LabelSpanSuffix` (new marker components); existing single-text label entity approach is replaced
- [x] 4.2 Add system `update_card_label_highlights` in `card.rs`: runs after `update_card_labels`; reads `CardMatchHighlight` and scenario data; when query is empty, sets all three spans to the same text as before (prefix = full text, highlight + suffix empty) and suffix/highlight colors to normal; when a range is present, splits the string at byte offsets and assigns prefix/highlight/suffix spans with `colors::YELLOW` for the highlight span; add `#[tracing::instrument(skip_all)]`
- [x] 4.3 Update `update_card_labels` to write only to the prefix span (or a temporary whole-text span) and skip highlight logic — that is now delegated to `update_card_label_highlights`

## 5. Empty State — Search-Specific Variant

- [x] 5.1 Add `EmptyStateSearchMessage` and `EmptyStateClearSearchButton` marker components to `empty_state.rs`
- [x] 5.2 In `spawn_empty_state`, add a second sub-group inside `EmptyStateNode`: a `Text` node for `"No scenarios match \"...\"` tagged `EmptyStateSearchMessage` (initial display `None`) plus a `"Clear Search"` button tagged `EmptyStateClearSearchButton` (initial display `None`)
- [x] 5.3 Rewrite `toggle_empty_state` to also accept `Res<SearchQuery>` and react to `search.is_changed()`; compute `visible_count` against both status filter and fuzzy match; when `visible_count == 0 && !search.0.is_empty()` show the search-empty sub-group and hide the generic sub-group; when `visible_count == 0 && search.0.is_empty()` show the generic sub-group; when `visible_count > 0` hide the whole `EmptyStateNode`; update the `EmptyStateSearchMessage` text to include the current query string
- [x] 5.4 Add system `handle_empty_clear_search_click` in `empty_state.rs`: when `EmptyStateClearSearchButton` `Interaction::Pressed`, set `SearchQuery("")` and `SearchFocused(false)`; add `#[tracing::instrument(skip_all)]`

## 6. System Registration

- [x] 6.1 In `mod.rs`, import and add the new systems to the `Update` schedule tuple: `handle_search_field_click`, `update_search_field_visuals`, `handle_search_clear_click`, `update_card_label_highlights`, `handle_empty_clear_search_click`
- [x] 6.2 Ensure ordering: `handle_text_search_input` runs before `apply_filter_and_sort`; `apply_filter_and_sort` runs before `update_card_label_highlights`; `update_card_labels` runs before `update_card_label_highlights`; use `.before()` / `.after()` constraints or extend the existing `.chain()` groups as appropriate
- [x] 6.3 Register `SearchFocused` resource init in `ExplorerViewPlugin::build` via `app.init_resource::<SearchFocused>()`

## 7. Verification

- [x] 7.1 Run `just check` and fix all compiler errors and clippy warnings
- [x] 7.2 Run `just lint` (clippy-tracing span check) — ensure every new public function has `#[tracing::instrument]`
- [x] 7.3 Run `just test` — confirm no regressions in existing explorer tests
- [x] 7.4 Run `just fmt` to apply nightly rustfmt import ordering
