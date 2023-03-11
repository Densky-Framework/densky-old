import { HTTPRequest, IController } from "densky";

export default class Controller implements IController {
  GET(req: HTTPRequest) {
    if (req.url.searchParams.has("mid")) {
      return new Response("MIDDLEWARE: Matched " + req.pathname);
    }
  }
}
