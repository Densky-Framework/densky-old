import { relative as relativePath } from "https://deno.land/std@0.190.0/path/mod.ts";
import { join } from "https://deno.land/std@0.190.0/path/posix.ts";

import { Globals } from "densky/cloud";
import { HTTPRequest } from "./request.ts";
import { HTTPResponse } from "./response.ts";

type ControllerResolved = {
  middlewares: Array<string>;
  fallbacks: Array<string>;
  controller: string;
};
type ManifestResolver = (req: HTTPRequest) => ControllerResolved | null;

export class DevServer extends BaseServer {
  lastId = 0;
  waitingRequests = new Map<number, Deno.RequestEvent>();

  manifestResolver: ManifestResolver | null = null;
  cache = new Map<string, EntryController>();

  constructor(
    options: BaseServerOptions,
    compileOptions: CompileOptions,
  ) {
    super(options);
    if (compileOptions.viewsPath) {
      HTTPResponse.viewsPath = join(Globals.cwd, "/.densky/views");
    }
  }

  importWithoutCache(url: string): Promise<unknown> {
    log("./" + stripPrefix(url, Globals.cwd), "CACHE", "import");
    return import(
      "file://" + url + "?k=" +
        (Math.random() * 16000 | 0).toString(32)
    );
  }

  async importController(url: string): Promise<EntryController | null> {
    if (this.cache.has(url)) {
      return this.cache.get(url)!;
    }

    try {
      const controller = await this.importWithoutCache(url) as EntryController;
      this.cache.set(url, controller);

      return controller;
    } catch (e) {
      log_error(e);
      return null;
    }
  }

  async handleRequest(req: HTTPRequest) {
    if (!this.manifestResolver) {
      type ManifestResolverModule = { default: ManifestResolver };

      this.manifestResolver = await this.importWithoutCache(
        join(Globals.cwd, ".densky/manifest.ts"),
      ).then((m) => (m as ManifestResolverModule).default);
    }

    if (req.pathname === "/$/dev") {
      this.manifestResolver = null;
      for (const entry of await req.raw.json()) {
        this.cache.delete(entry[1]);
        log(
          relativePath(Globals.cwd, entry[1]),
          "WATCHER",
          entry[0].toUpperCase(),
        );
      }

      return (new Response("Updated!", { status: 200 }));
    }

    const resolvedController = this.manifestResolver!(req);
    if (resolvedController == null) {
      return new Response("Not Found", { status: 404 });
    }

    for (const middleware of resolvedController.middlewares) {
      const out = await this.handleController(middleware, req);
      if (out) return out;
    }

    for (const fallback of resolvedController.fallbacks) {
      const out = await this.handleController(fallback, req);
      if (out) return out;
    }

    const out = await this.handleController(resolvedController.controller, req);
    if (out) return out;

    return new Response("Not found", { status: 404 });
  }

  async handleController(
    path: string,
    req: HTTPRequest,
  ): Promise<Response | null> {
    const controller = await this.importController(path);
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
}
