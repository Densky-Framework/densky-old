import { PrimitiveObject, StatusCode, statusMessages } from "../types.ts";
import color from "../colors.ts";

export class HTTPError {
  readonly message: string;

  name?: string;
  details?: PrimitiveObject;

  constructor(statusCode: StatusCode, message?: string);
  constructor(statusCode: number, message: string);
  constructor(readonly statusCode: number, message?: string) {
    if (message === undefined) {
      this.message = statusMessages[statusCode as StatusCode] ?? "";
    } else {
      this.message = message;
    }
  }

  withName(name?: string): this {
    this.name = name;
    return this;
  }

  withDetails(details?: PrimitiveObject): this {
    this.details = details;
    return this;
  }

  toJSON() {
    return {
      name: this.name,
      message: this.message,
      details: this.details,
    };
  }

  toResponse(): Response {
    return Response.json(this.toJSON(), {
      status: this.statusCode,
      statusText: statusMessages[this.statusCode as StatusCode] ?? this.message,
    });
  }

  static fromError(
    error: Error,
    statusCode: number | StatusCode = StatusCode.INTERNAL_ERR,
  ): HTTPError {
    // console.error(color.red`[HTTP ${statusCode}] ` + error.message);
    console.error(color.red`[HTTP ${statusCode}]`,error);

    return new HTTPError(statusCode, error.message)
      .withName(error.name)
      .withDetails({
        stack: error.stack ?? "",
      });
  }
}
