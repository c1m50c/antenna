<script lang="ts">
    import type { Query } from "../root.svelte";

    export let query: Query;
</script>

<section class="flex flex-col text-center">
    {query.name}
</section>
