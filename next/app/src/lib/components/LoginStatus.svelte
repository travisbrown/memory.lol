<script lang="ts">
  import { onMount } from "svelte";
  import {
    fetchLoginStatus,
    loginUrl,
    logoutUrl,
    type LoginStatus,
    type ProviderStatus,
  } from "$lib/api";
  import ProviderIcon from "./ProviderIcon.svelte";

  const PROVIDERS = [
    { key: "github", label: "GitHub" },
    { key: "google", label: "Google" },
    { key: "twitter", label: "Twitter" },
  ] as const;

  let status = $state<LoginStatus>({
    github: null,
    google: null,
    twitter: null,
  });
  let anySignedIn = $derived(PROVIDERS.some(({ key }) => status[key] !== null));

  onMount(async () => {
    try {
      status = await fetchLoginStatus();
    } catch {
      // An unreachable API just leaves the signed-out buttons in place.
    }
  });

  function rememberLocation() {
    window.sessionStorage.setItem("redirect", window.location.pathname);
  }

  function login(provider: string) {
    rememberLocation();
    window.location.assign(loginUrl(provider));
  }

  function logout() {
    rememberLocation();
    window.location.assign(logoutUrl());
  }

  const pillClass = (provider: ProviderStatus) =>
    provider.access.includes("trusted")
      ? "border-brand-green/60 bg-brand-green/15 text-brand-green"
      : "border-brand-yellow/60 bg-brand-yellow/15 text-brand-orange";
</script>

<div class="flex flex-wrap items-center gap-2">
  {#each PROVIDERS as { key, label } (key)}
    {@const provider = status[key]}
    {#if provider}
      <span
        class="inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-sm font-medium {pillClass(
          provider,
        )}"
        title="Signed in with {label}"
      >
        <ProviderIcon name={key} />
        {provider.name}
      </span>
    {:else}
      <button
        type="button"
        onclick={() => login(key)}
        class="inline-flex cursor-pointer items-center gap-2 rounded-full border border-edge bg-surface-raised px-3 py-1.5 text-sm font-medium text-ink-muted transition hover:border-ink-muted hover:text-ink"
      >
        <ProviderIcon name={key} />
        Sign in with {label}
      </button>
    {/if}
  {/each}
  {#if anySignedIn}
    <button
      type="button"
      onclick={logout}
      class="inline-flex cursor-pointer items-center gap-2 rounded-full border border-red-400/50 bg-red-400/10 px-3 py-1.5 text-sm font-medium text-red-500 transition hover:bg-red-400/20"
    >
      Sign out
    </button>
  {/if}
</div>
