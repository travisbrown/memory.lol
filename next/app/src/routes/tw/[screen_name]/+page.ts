import { error } from "@sveltejs/kit";
import { lookupScreenName, toEntries } from "$lib/api";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ params, fetch }) => {
  try {
    return {
      query: params.screen_name,
      entries: toEntries(await lookupScreenName(params.screen_name, fetch)),
    };
  } catch (cause) {
    error(502, `Screen name lookup failed: ${cause}`);
  }
};
