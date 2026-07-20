<script lang="ts">
  import { base } from "$app/paths";
  import type { ResultEntry } from "$lib/api";

  let { entries }: { entries: ResultEntry[] } = $props();

  const dateRange = (dates: string[] | null): [string, string] => {
    if (!dates || dates.length === 0) {
      return ["unknown", "unknown"];
    }
    return [dates[0], dates[dates.length - 1]];
  };

  const linkClass =
    "text-brand-green underline decoration-brand-green/40 underline-offset-2 hover:decoration-brand-green";
</script>

<div class="flex flex-col gap-6">
  {#each entries as [query, account] (`${query ?? ""}-${account.id_str}`)}
    <section
      class="overflow-hidden rounded-xl border border-edge bg-surface-raised shadow-sm"
    >
      <h2
        class="border-b border-edge px-4 py-3 font-display text-sm font-semibold tracking-wide"
      >
        {account.id_str}{#if query}
          <span class="text-ink-muted">({query})</span>
        {/if}
      </h2>
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr
              class="text-left text-xs tracking-wider text-ink-muted uppercase"
            >
              <th class="px-4 py-2 font-medium">Screen name</th>
              <th class="px-4 py-2 font-medium">First seen</th>
              <th class="px-4 py-2 font-medium">Last seen</th>
              <th class="px-4 py-2 font-medium" colspan="3">Links</th>
            </tr>
          </thead>
          <tbody>
            {#each Object.entries(account.screen_names) as [screenName, dates] (screenName)}
              {@const [first, last] = dateRange(dates)}
              <tr class="border-t border-edge/60 hover:bg-surface">
                <td class="px-4 py-2 font-medium">
                  <a
                    href="{base}/tw/{encodeURIComponent(screenName)}"
                    class={linkClass}
                  >
                    {screenName}
                  </a>
                </td>
                <td class="px-4 py-2 tabular-nums">{first}</td>
                <td class="px-4 py-2 tabular-nums">{last}</td>
                <td class="px-4 py-2">
                  <a
                    href="https://twitter.com/intent/user?user_id={account.id_str}"
                    class={linkClass}
                  >
                    Current Twitter ID
                  </a>
                </td>
                <td class="px-4 py-2">
                  <a href="https://twitter.com/{screenName}" class={linkClass}>
                    Current screen name
                  </a>
                </td>
                <td class="px-4 py-2">
                  <a
                    href="https://web.archive.org/web/*/https://twitter.com/{screenName}/status/*"
                    class={linkClass}
                  >
                    Wayback Machine
                  </a>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>
  {:else}
    <p
      class="rounded-xl border border-edge bg-surface-raised px-4 py-6 text-center text-ink-muted"
    >
      No results found.
    </p>
  {/each}
</div>
