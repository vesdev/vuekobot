<script lang="ts">
	import "../global.css";
</script>

<header/>
<main>
	<slot />
</main>

<style>
	:root {
		background-color: #16161c;
	}

	main {
    font-family: "ApfelGrotezk";
    font-style: normal;
		width: 100%;
		height: 100%;
	}

	header {
		height: 50px;
		background-color: #16161c;
	}

</style>
