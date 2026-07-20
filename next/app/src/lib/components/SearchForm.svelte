<script lang="ts">
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import SearchField from "./SearchField.svelte";

  let screenName = $state("");
  let twitterId = $state("");

  function searchScreenName(event: SubmitEvent) {
    event.preventDefault();
    if (screenName !== "") {
      goto(`${base}/tw/${encodeURIComponent(screenName.trim())}`);
    }
  }

  function searchTwitterId(event: SubmitEvent) {
    event.preventDefault();
    if (twitterId !== "") {
      goto(`${base}/tw/id/${encodeURIComponent(twitterId.trim())}`);
    }
  }
</script>

<search class="flex flex-col gap-3">
  <SearchField
    bind:value={screenName}
    name="screen_name"
    placeholder="Screen name (comma-separated, or prefix*)"
    onsubmit={searchScreenName}
  />
  <SearchField
    bind:value={twitterId}
    name="twitter_id"
    placeholder="Twitter ID"
    inputmode="numeric"
    onsubmit={searchTwitterId}
  />
</search>
