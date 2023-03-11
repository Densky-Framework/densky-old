import { HTTPRequest, IController } from "densky";

export default class CookiesController implements IController {
  GET(req: HTTPRequest) {
    return Response.json(req.cookies.raw);
  }
}
