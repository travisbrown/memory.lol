<script lang="ts">
  import { page } from "$app/state";
</script>

<div
  class="rounded-xl border border-red-400/40 bg-red-400/10 px-4 py-6 text-center"
>
  <p class="font-display text-lg font-semibold">{page.status}</p>
  <p class="mt-1 text-sm text-ink-muted">{page.error?.message}</p>
</div>
