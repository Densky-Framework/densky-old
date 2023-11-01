// Based on https://github.com/withastro/starlight/blob/main/packages/starlight/utils/routing.ts

import type { GetStaticPathsItem, MarkdownHeading } from "astro";
import { getCollection } from "astro:content";
import type { CollectionEntry } from "astro:content";
import { type SidebarEntry, treeify } from "./navigation";

export type DocsEntry = Omit<CollectionEntry<"docs">, "slug"> & {
  slug: string;
};

export interface Route {
  // /** Content collection entry for the current page. Includes frontmatter at `data`. */
  entry: DocsEntry;
  // entryMeta: LocaleData;
  slug: string;
  id: string;
  isFallback?: true;

  title: string;

  sidebar: {
    label: string;
    hidden: boolean;
    entries: SidebarEntry["children"];
  };
  [key: string]: unknown;
}

interface Path extends GetStaticPathsItem {
  params: { slug: string | undefined };
  props: Route;
}

const normalizeIndexSlug = (slug: string) => (slug === "index" ? "" : slug);

const docs = ((await getCollection("docs")) ?? []).map(
  ({ slug, ...entry }) => ({
    ...entry,
    slug: normalizeIndexSlug(slug),
  }),
);

function getRoutes(): Route[] {
  return docs.map((entry) => {
    return {
      entry,
      slug: slugToParam(entry.slug),
      id: entry.id,

      title: entry.data.title,

      sidebar: {
        label: entry.data.sidebar?.label ?? entry.data.title,
        hidden: entry.data.sidebar?.hidden ?? false,
        entries: [],
      },
      // entryMeta: slugToLocaleData(entry.slug),
      // ...slugToLocaleData(entry.slug),
    };
  });
}

const routes = getRoutes();

function getPaths(): Path[] {
  return routes.map((route) => ({
    params: { slug: route.slug || undefined },
    props: {
      ...route,
      sidebar: {
        ...route.sidebar,
        entries: treeify(routes, ""),
      },
    },
  }));
}
export const paths = getPaths();

export function slugToParam(slug: string): string {
  return (slug === "index" || slug === ""
    ? ""
    : slug.endsWith("/index")
    ? slug.replace(/\/index$/, "")
    : slug.replace(/\d+/, "").replaceAll(/\/\d+/g, "/"));
}

export type PageProps = Route & {
  headings: MarkdownHeading[];
};

export function generateRouteData(props: PageProps) {
  const breadcrumb = props.slug
    .split("/")
    .slice(0, -1)
    .map((p) => [
      p.replaceAll("-", " ").replace(/^\w/, (a) => a.toUpperCase()),
      p,
    ])
    .reduce<{ items: [string, string][]; carry: string }>((prev, [title, path]) => {
      prev.items.push([title, prev.carry + "/" + path]);
      return prev;
    }, { items: [], carry: "/docs" })
    .items;

  return {
    ...props,
    breadcrumb,
    sidebar: props.sidebar,
    // pagination: getPrevNextLinks(sidebar, config.pagination, entry.data),
    // toc: getToC(props),
    // lastUpdated: getLastUpdated(props),
    // editUrl: getEditUrl(props),
  };
}
