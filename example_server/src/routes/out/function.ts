import { IController } from "densky";
import { outFunction } from "../../outFunction.ts";

export default class OutFunctionController implements IController {
  GET() {
    return new Response(outFunction(1, 2).toString());
  }
}
