<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  let {
    value = $bindable(""),
    name,
    placeholder,
    inputmode,
    onsubmit,
  }: {
    value?: string;
    name: string;
    placeholder: string;
    inputmode?: HTMLInputAttributes["inputmode"];
    onsubmit: (event: SubmitEvent) => void;
  } = $props();

  let input = $state<HTMLInputElement>();

  function clear() {
    value = "";
    input?.focus();
  }

  // Escape-to-clear is native search input behavior in some browsers; handling
  // it here makes it work in all of them.
  function onkeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && value !== "") {
      event.preventDefault();
      clear();
    }
  }
</script>

<form {onsubmit} class="flex">
  <div class="relative grow">
    <input
      bind:this={input}
      bind:value
      type="search"
      {name}
      {placeholder}
      {inputmode}
      {onkeydown}
      class="w-full rounded-l-lg border border-r-0 border-edge bg-surface-raised py-2 pr-9 pl-3
        text-sm text-ink placeholder:text-ink-muted focus:border-brand-green focus:outline-none"
    />
    {#if value}
      <button
        type="button"
        aria-label="Clear search"
        onclick={clear}
        class="absolute inset-y-0 right-1.5 my-auto flex h-6 w-6 cursor-pointer items-center
          justify-center rounded-full text-ink-muted transition hover:bg-edge/60 hover:text-ink"
      >
        <svg viewBox="0 0 20 20" class="h-3.5 w-3.5" aria-hidden="true">
          <path
            d="M6 6l8 8M14 6l-8 8"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
          />
        </svg>
      </button>
    {/if}
  </div>
  <button
    type="submit"
    class="cursor-pointer rounded-r-lg bg-brand-green px-4 py-2 text-sm font-semibold text-white
      transition hover:brightness-105 active:brightness-95"
  >
    Search
  </button>
</form>
