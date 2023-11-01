// Based on https://github.com/withastro/starlight/blob/main/packages/starlight/utils/navigation.ts

import { basename, dirname } from "node:path";
import type { Route } from "./routing";

export interface SidebarEntry {
  type: "dir";
  label: string;
  children: Array<
    SidebarEntry | (Route & { sidebar: Omit<Route["sidebar"], "entries"> })
  >;
}

const stripExtension = (path: string) => path.replace(/\.\w+$/, "");

/** Get the segments leading to a page. */
function getBreadcrumbs(path: string, baseDir: string): string[] {
  const pathWithoutExt = stripExtension(path);
  if (pathWithoutExt === baseDir) return [];
  if (!baseDir.endsWith("/")) baseDir += "/";

  const relativePath = pathWithoutExt.startsWith(baseDir)
    ? pathWithoutExt.replace(baseDir, "")
    : pathWithoutExt;
  let dir = dirname(relativePath);

  if (dir === ".") return [];
  return dir.split("/");
}

type A = Map<string, A | Route>;

const treeify_normalize = (
  [path, children]: [string, Route | A],
): SidebarEntry => ({
  type: "dir",
  label: path,
  children:
    (children instanceof Map
      ? [...((children as A).entries())].map((entry) =>
        entry[1] instanceof Map ? treeify_normalize(entry) : entry[1] as Route
      )
      : [children as Route]),
});

/** Turn a flat array of routes into a tree structure. */
export function treeify(
  routes: Route[],
  baseDir: string,
): SidebarEntry["children"] {
  const treeRoot: A = new Map();
  routes
    .filter((doc) => !doc.sidebar.hidden)
    .forEach((doc) => {
      const breadcrumbs = getBreadcrumbs(doc.id, baseDir);

      let currentDir = treeRoot;
      breadcrumbs.forEach((dir) => {
        if (!currentDir.has(dir)) currentDir.set(dir, new Map());
        const a = currentDir.get(dir);
        if (!a || !(a instanceof Map)) throw new Error("unreachable");

        currentDir = a;
      });
      currentDir.set(basename(stripExtension(doc.id)), doc);
    });

  const b = [...treeRoot.entries()].map((entry) => treeify_normalize(entry));
  return b;
}

/** Turn the nested tree structure of a sidebar into a flat list of all the links. */
export function flattenSidebar(sidebar: SidebarEntry["children"]): Route[] {
  return sidebar.flatMap((entry) =>
    "type" in entry ? flattenSidebar((entry as SidebarEntry).children) : entry
  );
}

/** Get previous/next pages in the sidebar or the ones from the frontmatter if any. */
export function getPrevNextLinks(sidebar: SidebarEntry["children"], url: string): {
  /** Link to previous page in the sidebar. */
  prev: Route | undefined;
  /** Link to next page in the sidebar. */
  next: Route | undefined;
} {
  const entries = flattenSidebar(sidebar);
  const currentIndex = entries.findIndex((entry) => "/docs/" + entry.slug === url);
  const prev = entries[currentIndex - 1];
  const next = currentIndex > -1 ? entries[currentIndex + 1] : undefined;
  return { prev, next };
}
