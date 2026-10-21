---
sources: [.tickets/closed/TCK-115-master-architectural-refactoring.md, .tickets/closed/TCK-125-dry-kiss-cleanup.md, .tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, .tickets/closed/FIX-131-hox-symmetry-visuals-reef-collision.md, .tickets/closed/TCK-141-frontend-hygiene.md]
---

# Frontend History (implemented)

| Ticket | Date | What was implemented | Article |
| :--- | :--- | :--- | :--- |
| TCK-115 | 2026-08-22 | SPA with signal-driven view switch, `InteractiveCamera`, shared `BrainRenderer`, Vite config reduced to a plain SPA bundler | [[frontend/structure]] |
| TCK-125 | 2026-08-31 | Frontend map generator removed (world comes from Rust) | [[simulation/ocean]] |
| TCK-128 | 2026-09-03 | Main menu and router; `styled.ts`; `UnifiedDiagnosticsPanel` with 5 modules; catalogue view, cryo-cloning and restore selector; catalogue DB table and 5 commands | [[frontend/structure]], [[architecture/persistence]] |
| FIX-131 | 2026-09-03 | Organelles rendered as smooth receptor domes (`wAngular = 0.18 + bandwidth · 0.22`) instead of pins | [[frontend/structure]] |
| TCK-141 | 2026-10-19 | `escapeHtml` for DB/user data, `zustand` removed, engine logs unknown actions (RV-F02, RV-F07, RV-B09) | [[frontend/structure]] |

## Key Takeaways

- The current UI shape (menu, four views, unified diagnostics panel) dates from TCK-128.
- Output escaping is in place; CSP and view lifecycle are still open.

Sources: closed tickets listed in the frontmatter.
