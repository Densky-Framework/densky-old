import { relative as relativePath } from "https://deno.land/std@0.190.0/path/mod.ts";
import { join } from "https://deno.land/std@0.190.0/path/posix.ts";

import {
  BaseServer,
  DevServer,
  Globals,
  Logger,
  ServerCache,
  stripPrefix,
} from "densky/cloud.ts";
import { HTTPRequest } from "./request.ts";
import { HTTPResponse } from "./response.ts";
import { EntryController } from "./types.ts";

type ControllerResolved = {
  middlewares: Array<string>;
  fallbacks: Array<string>;
  controller: string;
};
type ManifestResolver = (req: HTTPRequest) => ControllerResolved | null;

const logger = new Logger("HTTP");
const logger_cache = new Logger("HTTP:CACHE");

let manifestResolver: ManifestResolver | null = null;
const cache = new ServerCache<EntryController>();

function importWithoutCache(url: string): Promise<unknown> {
  logger_cache.debug("import ", "./" + stripPrefix(url, Globals.cwd));
  return import(
    "file://" + url + "?k=" +
      (Math.random() * 16000 | 0).toString(32)
  );
}

async function importController(url: string): Promise<EntryController | null> {
  const _ = cache.get(url);
  if (_) {
    return _!;
  }

  try {
    const controller = await importWithoutCache(url) as EntryController;
    cache.set(url, controller);

    return controller;
  } catch (e) {
    logger_cache.error(e);
    return null;
  }
}

async function handleController(
  path: string,
  req: HTTPRequest,
): Promise<Response | null> {
  const controller = await importController(path);
  if (!controller) {
    return new Response("Controller doesn't exist: " + path, {
      status: 500,
    });
  }

  const entry = controller[req.method as keyof EntryController] ??
    controller.default;
  if (!entry) return new Response("Method not implemented", { status: 402 });

  await req.prepare();
  const out = await entry(req);
  if (out == null) return null;

  return HTTPResponse.toResponse(req, out);
}

DevServer.hooks.registerHook("beforeWatchUpdate", () => {
  manifestResolver = null;
});

DevServer.hooks.registerHook("watchUpdate", (kind, path) => {
  cache.delete(path);
  logger.info(
    kind.toUpperCase(),
    " WATCHER ",
    relativePath(Globals.cwd, path),
  );
});

BaseServer.hooks.registerHook("request", async (r) => {
  {
    const u = new URL(r.url);
    if (u.pathname.startsWith("/$")) return null;
  }

  const req = new HTTPRequest(r);

  if (!manifestResolver) {
    type ManifestResolverModule = { default: ManifestResolver };

    manifestResolver = await importWithoutCache(
      join(Globals.cwd, ".densky/manifest.ts"),
    ).then((m) => (m as ManifestResolverModule).default);
  }

  const resolvedController = manifestResolver!(req);
  if (resolvedController == null) {
    return new Response("Not Found", { status: 404 });
  }

  for (const middleware of resolvedController.middlewares) {
    const out = await handleController(middleware, req);
    if (out) return out;
  }

  for (const fallback of resolvedController.fallbacks) {
    const out = await handleController(fallback, req);
    if (out) return out;
  }

  const out = await handleController(resolvedController.controller, req);
  if (out) return out;

  return new Response("Not found", { status: 404 });
});
