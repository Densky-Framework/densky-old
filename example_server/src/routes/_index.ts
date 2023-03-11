import { HTTPResponse, IController } from "densky";

export default class _ implements IController {
  async GET() {
    return await HTTPResponse.view("index.html");
  }
}
