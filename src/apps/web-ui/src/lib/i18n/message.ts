// Message model for co-located, typed UI copy (plan D3).
// `*.messages.ts` files import only this module, so `bun test` can load them.

export type Locale = "en" | "tr";

export const LOCALES: readonly Locale[] = ["en", "tr"];

export const DEFAULT_LOCALE: Locale = "en";

export type MessageParams = Record<string, string | number>;

// `any` keeps per-message parameter types (e.g. `{ count: number }`) assignable.
export type MessageFunction<P extends MessageParams = any> = (
  params: P,
) => string;

/** A string with `{param}` placeholders, given in every locale. */
export type StringMessage = { readonly en: string; readonly tr: string };

/** A function pair for plurals and Turkish suffixes (use `Intl.PluralRules`). */
export type FunctionMessage<P extends MessageParams = any> = {
  readonly en: MessageFunction<P>;
  readonly tr: MessageFunction<P>;
};

export type Message = StringMessage | FunctionMessage;

export type MessageParamsOf<M> =
  M extends FunctionMessage<infer P> ? P : MessageParams;

export function isLocale(value: unknown): value is Locale {
  return value === "en" || value === "tr";
}

/** Declares a typed record of messages; `tr` is required by the type. */
export function defineMessages<const T extends Record<string, Message>>(
  messages: T,
): T {
  return messages;
}
