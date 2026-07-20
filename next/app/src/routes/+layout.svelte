<script lang="ts">
  import "../app.css";
  import { onMount, type Snippet } from "svelte";
  import { goto } from "$app/navigation";
  import { assets } from "$app/paths";
  import LoginStatus from "$lib/components/LoginStatus.svelte";
  import SearchForm from "$lib/components/SearchForm.svelte";

  let { children }: { children: Snippet } = $props();

  // Complete the round trip started by a login/logout redirect: return to the
  // page the user was on when they clicked the button.
  onMount(() => {
    const redirect = window.sessionStorage.getItem("redirect");

    if (redirect) {
      window.sessionStorage.removeItem("redirect");
      goto(redirect, { replaceState: true });
    }
  });
</script>

<div class="flex min-h-screen flex-col">
  <header class="border-b border-edge bg-surface-raised">
    <nav class="mx-auto flex max-w-5xl flex-wrap items-center gap-4 px-4 py-3">
      <a href="https://memory.lol/" class="shrink-0">
        <img alt="memory.lol" src="{assets}/logos/dumpster.svg" class="h-12" />
      </a>
      <a
        href="https://github.com/travisbrown/memory.lol"
        class="text-sm font-medium text-ink-muted hover:text-ink"
      >
        About
      </a>
      <div class="ms-auto">
        <LoginStatus />
      </div>
    </nav>
  </header>

  <main class="mx-auto w-full max-w-5xl grow px-4 py-8">
    <div class="grid gap-8 md:grid-cols-2">
      <section>
        <h1 class="font-display text-2xl font-semibold">
          Welcome to memory.lol
        </h1>
        <div class="mt-3 space-y-3 text-sm leading-relaxed text-ink-muted">
          <p>
            This site is an instance of software from the
            <a
              href="https://github.com/travisbrown/hassreden-tracker"
              class="font-medium text-ink underline underline-offset-2"
            >
              Hassreden-Tracker
            </a>
            project. Search results are limited for unauthenticated users (see
            <a
              href="https://github.com/travisbrown/memory.lol#current-access-restrictions"
              class="font-medium text-ink underline underline-offset-2"
            >
              this document
            </a>
            for details).
          </p>
          <p>
            Please contact us
            <a
              href="mailto:travisrobertbrown@protonmail.com"
              class="font-medium text-ink underline underline-offset-2"
            >
              by email
            </a>
            to discuss trusted access.
          </p>
        </div>
      </section>
      <section>
        <h2 class="font-display text-2xl font-semibold">
          Twitter history search
        </h2>
        <div class="mt-3">
          <SearchForm />
        </div>
      </section>
    </div>

    <div class="mt-8">
      {@render children()}
    </div>
  </main>

  <footer class="border-t border-edge bg-surface-raised">
    <p class="mx-auto max-w-5xl px-4 py-6 text-center text-sm text-ink-muted">
      <a
        href="https://github.com/travisbrown/hassreden-tracker"
        class="font-semibold text-ink"
      >
        Hassreden-Tracker
      </a>
      and
      <a href="https://memory.lol/app/" class="font-semibold text-ink"
        >memory.lol</a
      >
      are developed by
      <a
        href="https://twitter.com/travisbrown"
        class="underline underline-offset-2">Travis Brown</a
      >. The source code is licensed under the
      <a
        href="https://anticapitalist.software/"
        class="underline underline-offset-2"
      >
        Anti-Capitalist Software License</a
      >.
    </p>
  </footer>
</div>
