<script lang="ts" context="module">
    export type Query = {
        name: string;
        path: string;
        matches: Match[];
    };

    export type Match = {
        captures: Capture[];
    };

    export type Capture = {
        text: string;
        name: string;
        start_column: number;
        start_line: number;
        end_column: number;
        end_line: number;
    };
</script>

<script lang="ts">
    import QueryCard from "./components/query_card.svelte";

    const outQueries = Object.values(
        import.meta.glob("./queries/**/*.json", { eager: true }),
    )
        .map((x) => (x as { default: Query[] }).default)
        .flat();
</script>

<div>
    <h1 class="text-3xl font-semibold">Antenna</h1>

    {#each outQueries as outQuery}
        <QueryCard query={outQuery} />
    {/each}
</div>
