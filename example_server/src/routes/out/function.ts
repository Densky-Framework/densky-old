import { outFunction } from "../../outFunction.ts";

export function GET() {
  return new Response(outFunction(1, 2).toString());
}
