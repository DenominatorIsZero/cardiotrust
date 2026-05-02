## 1. Remove Legacy Backend Wiring

- [x] 1.1 Remove `UiType` state, its defaulting logic, and the F2 backend-toggle system from the UI bootstrap.
- [x] 1.2 Update UI plugin registration and any state-gated systems so the supported Bevy-native path runs without backend branching.
- [x] 1.3 Update inline comments or developer-facing UI docs in code that still describe the deleted alternate backend.

## 2. Delete Legacy Egui UI Modules

- [x] 2.1 Remove the legacy egui modules for the old top bar, explorer, scenario editor, and volumetric screen from `src/ui/`.
- [x] 2.2 Remove imports, registrations, and helper code that existed only to support those deleted modules.
- [x] 2.3 Verify the retained Bevy-native views still cover Home, Explorer, Scenario, Results, Volumetric, and Scheduler behavior.

## 3. Clean Up Dependencies

- [x] 3.1 Inventory crate usage after the code deletions and remove only dependencies that no longer have live references.
- [x] 3.2 Keep scoped overlay and file-dialog dependencies that are still required by supported Bevy-native flows.
- [x] 3.3 Run formatting after manifest and source cleanup.

## 4. Verify Single-Path Behavior

- [x] 4.1 Run the relevant checks/tests for the UI cleanup and fix any fallout from deleted backend-branching code.
- [x] 4.2 Verify the application starts on Home and that navigation no longer promises a legacy top bar or runtime backend switching.
- [x] 4.3 Confirm the retained volumetric overlay and project-loading flows still work after dependency cleanup.
