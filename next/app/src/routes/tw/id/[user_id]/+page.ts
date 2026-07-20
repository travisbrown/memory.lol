import { error } from "@sveltejs/kit";
import { lookupUserId, toEntries } from "$lib/api";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ params, fetch }) => {
  try {
    return {
      query: params.user_id,
      entries: toEntries(await lookupUserId(params.user_id, fetch)),
    };
  } catch (cause) {
    error(502, `User ID lookup failed: ${cause}`);
  }
};
