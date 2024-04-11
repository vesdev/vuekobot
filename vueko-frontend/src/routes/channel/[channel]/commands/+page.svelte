<script lang="ts">
	import { page } from '$app/stores';
	export let data;
</script>

<div class="command-list">
	<h1><span class="channel">#{$page.params.channel}</span> commands</h1>

	<table>
		<thead>
			<th>command</th>
			<th>value</th>
		</thead>
		<tbody>
			{#each data.commands as command}
				<tr>
					<td class="command">!{command.command}</td>
					<td class="value">{command.value}</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<style>
	h1 {
		padding-top: 40px;
		text-align: center;
		color: #d5d8da;
		font-weight: 100;
	}

	table {
		table-layout: fixed;
		border-spacing: 20px 20px;
		width: 100%;
		max-width: 1000px;
	}

	tr,
	td {
		padding: 20px;
		border: 1px solid #353747;
	}

	th {
		color: #e95678;
		font-size: 1.5rem;
		font-weight: 100;
	}

	.command-list {
		display: flex;
		flex-direction: column;
		align-items: center;
		width: 100%;
		background-color: #1c1e26;
	}

	.channel {
		color: #FAB795;
	}

	.command {
		color: #d5d8da;
	}

	.value {
		color: #d5d8da;
	}
</style>
