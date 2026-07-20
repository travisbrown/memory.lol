<script lang="ts">
  import ResultsTable from "$lib/components/ResultsTable.svelte";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
</script>

<svelte:head>
  <title>{data.query} - memory.lol</title>
</svelte:head>

<ResultsTable entries={data.entries} />
