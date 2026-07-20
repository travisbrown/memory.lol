import { env } from "$env/dynamic/public";

/**
 * Root of the JSON API. Empty (same origin) by default, in which case requests
 * go through the /v1 reverse proxy; set PUBLIC_API_ROOT at build time to target
 * another host.
 */
export const API_ROOT: string = env.PUBLIC_API_ROOT ?? "";

export interface ProviderStatus {
  id: string;
  name: string;
  access: string[];
}

export interface LoginStatus {
  github: ProviderStatus | null;
  google: ProviderStatus | null;
  twitter: ProviderStatus | null;
}

export interface AccountResult {
  id: number;
  id_str: string;
  /** Screen names mapped to [first, last] observation dates (or null if unknown). */
  screen_names: Record<string, string[] | null>;
}

export interface AccountsResult {
  accounts: AccountResult[];
}

/** A screen name lookup result: either one result set or a map keyed by query. */
export type ScreenNameResult = AccountsResult | Record<string, AccountsResult>;

/** A result row ready for display: the matched query (if any) and the account. */
export type ResultEntry = [string | null, AccountResult];

const isAccountsResult = (value: ScreenNameResult): value is AccountsResult =>
  Array.isArray((value as AccountsResult).accounts);

/** Flatten any lookup response shape into displayable [query, account] entries. */
export function toEntries(
  result: AccountResult | ScreenNameResult,
): ResultEntry[] {
  if ("id_str" in result) {
    return [[null, result as AccountResult]];
  }

  if (isAccountsResult(result)) {
    return result.accounts.map((account) => [null, account]);
  }

  return Object.entries(result).flatMap(([query, { accounts }]) =>
    accounts.map((account): ResultEntry => [query, account]),
  );
}

async function getJson<T>(path: string, fetcher: typeof fetch): Promise<T> {
  const response = await fetcher(`${API_ROOT}/v1${path}`, {
    credentials: "include",
  });

  if (!response.ok) {
    throw new Error(`API request failed with status ${response.status}`);
  }

  return response.json();
}

/** Fetch the signed-in status for all providers. */
export const fetchLoginStatus = (fetcher: typeof fetch = fetch) =>
  getJson<LoginStatus>("/login/status", fetcher);

/** Look up an account's screen name history by Twitter ID. */
export const lookupUserId = (userId: string, fetcher: typeof fetch = fetch) =>
  getJson<AccountResult>(`/tw/id/${encodeURIComponent(userId)}`, fetcher);

/** Look up accounts by screen name, comma-separated list, or `prefix*`. */
export const lookupScreenName = (
  query: string,
  fetcher: typeof fetch = fetch,
) => getJson<ScreenNameResult>(`/tw/${encodeURIComponent(query)}`, fetcher);

/** The login redirect URL for a provider (`github`, `google`, or `twitter`). */
export const loginUrl = (provider: string) =>
  `${API_ROOT}/v1/login/${provider}`;

/** The logout redirect URL. */
export const logoutUrl = () => `${API_ROOT}/v1/logout`;
