<script lang="ts">
	import favicon from "$lib/assets/favicon.svg";
	import "../styles/globals.scss";

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Owlimatronic</title>
</svelte:head>

{@render children?.()}
