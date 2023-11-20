import type { Cookie } from "https://deno.land/std@0.160.0/http/cookie.ts";
import {
  decode as decode64,
  encode as encode64,
} from "https://deno.land/std@0.160.0/encoding/base64.ts";
import {
  getCookies,
  setCookie,
} from "https://deno.land/std@0.160.0/http/cookie.ts";
import { Logger } from "densky/cloud.ts";

export const logger = new Logger("HTTP:COOKIES");

export const cookiePasswordKey = "DENSKY_COOKIE_PASSWORD";

if (!Deno.env.get(cookiePasswordKey)) {
  logger.warn(
    "Cookie password is not setted, using fallback 'not-setted'. Env key: " +
      cookiePasswordKey,
  );
}

const cookiePassword = Deno.env.get(cookiePasswordKey) || "not-setted";
const cookieKey = await crypto.subtle.importKey(
  "raw",
  new TextEncoder().encode(cookiePassword),
  { name: "HMAC", hash: "SHA-256" },
  true,
  ["sign", "verify"],
);

const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8");

/** @internal */
export async function sign(data: string) {
  const d = await crypto.subtle.sign("HMAC", cookieKey, encoder.encode(data));

  return encode64(d);
}

/** @internal */
export async function verify(signature: string, data: string) {
  const sign_decoded = decode64(signature);

  return await crypto.subtle.verify(
    "HMAC",
    cookieKey,
    sign_decoded,
    encoder.encode(data),
  );
}

export async function getDecodedCookies(
  headers: Headers,
): Promise<Record<string, string>> {
  const cookies = getCookies(headers);

  for (const [key, cookie] of Object.entries(cookies)) {
    const isSigned = cookie.startsWith("s:");
    if (!isSigned) {
      cookies[key] = decodeURI(cookie);
      continue;
    }

    try {
      // The signed cookie is always in base64 and
      // has the next format: VALUE.SIGNATURE
      const [value, signature] = cookie.slice(2)
        .split(".", 2);

      const decodedValue = decoder.decode(decode64(value));

      const verified = await verify(signature, decodedValue);

      if (verified) {
        cookies[key] = decodedValue;
      } else {
        delete cookies[key];
      }
    } catch (e) {
      logger.warn("Error decoding cookie (" + key + "):", (e as Error).stack);
    }
  }

  return cookies;
}

/** @internal */
export async function signedCookie(data: string) {
  const signature = await sign(data);

  return `s:${encode64(data)}.${signature}`;
}

export type CookieOptions = Omit<Cookie, "name" | "value"> & {
  raw?: boolean;
};

export async function setEncodedCookie(
  headers: Headers,
  key: string,
  value: string,
  options: CookieOptions,
): Promise<void> {
  const cookieValue = options.raw
    ? encodeURI(value)
    : await signedCookie(value);

  const cookie: Cookie = {
    name: key,
    value: cookieValue,
    ...options,
  };

  setCookie(headers, cookie);
}
