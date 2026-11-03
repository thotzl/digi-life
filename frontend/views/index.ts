// View registry (TCK-160): the one place a view is added. Order = DOM order. Contract:
// wiki/contracts/frontend-view.md, guide wiki/guides/add-view.md.
import { catalogueView } from "./catalogue";
import { mainMenuView } from "./main-menu";
import { oceanView } from "./ocean";
import { trainerView } from "./trainer";

export const VIEWS = [mainMenuView, oceanView, trainerView, catalogueView] as const;

/** Ids of the registered views (formerly a hand-written union that also listed a "creator" view without a module). */
export type ViewId = (typeof VIEWS)[number]["id"];

export const INITIAL_VIEW: ViewId = "main-menu";
