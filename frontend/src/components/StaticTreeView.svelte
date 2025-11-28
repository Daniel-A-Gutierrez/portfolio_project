<!-- <script context="module">
	// retain module scoped expansion state for each tree node
	const _expansionState = {
		/* treeNodeId: expanded <boolean> */
	}
</script> -->
<script>
	export let tree;
	const {label, children} = tree;

	let expanded = true; // _expansionState[label] || false;
	// const toggleExpansion = () => {
	// 	expanded = _expansionState[label] = !expanded
	// }
	$: arrowDown = expanded;
</script>

<ul><!-- transition:slide -->
	<li>
		{#if children}
			<span>
				<span class="arrow" class:arrowDown>&#x25b6</span>
				{label}
			</span>
			{#if expanded}
				{#each children as child}
					<svelte:self tree={child} />
				{/each}
			{/if}
		{:else}
			<span>
				<span class="no-arrow">
					{label}
				</span>
			</span>
		{/if}
	</li>
</ul>

<style>
	span 
	{
		font-family: monospace;
		font-size: 1.125rem;
		line-height: 1.125rem;
	}
	ul {
		margin: 0;
		list-style: none;
		padding-left: 1.08rem; 
	}
	.no-arrow { 
		padding-left: 1rem; 
	}
	.no-arrow::before 
	{
		content:"- ";
	}
	.arrow {
		display: inline-block;
		transform: translate(0px, -2px) scale(.6);
	}
	.arrowDown { transform: scale(.6) rotate(90deg); }
</style>
