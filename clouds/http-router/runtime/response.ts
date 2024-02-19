import { HTTPRequest } from "./request.ts";
import { HTTPError } from "./error.ts";

export class HTTPResponse {
  static viewsPath: string;

  static async view(
    path: string,
    data?: unknown,
    init?: ResponseInit,
  ): Promise<Response> {
    return new Response(
      JSON.stringify({
        view_path: path,
        data: data,
      }),
      {
        status: 200,
        ...init,
        headers: {
          "Content-Type": "application/json",
          ...init?.headers,
        },
      },
    );
  }

  static toResponse(
    req: HTTPRequest,
    response: Response | HTTPError | Error | void,
  ): Response {
    if (response instanceof Error) {
      response = HTTPError.fromError(response);
    }

    if (response instanceof HTTPError) {
      response = response.toResponse();
    }

    if (response instanceof Response) {
      return new Response(response.body, {
        status: response.status,
        statusText: response.statusText,
        headers: Object.fromEntries([
          ...req.headers.entries(),
          ...response.headers.entries(),
        ]),
      });
    }

    throw new Error("Unreachable code");
  }
}
