# store

automagically synced svelte stores

this is a **WIP**, currently just a proof of concept and only supports svelte stores.

**example usage**
in this example, we have a namespace `nathan` with two stores, `counter` and `input`. the `counter` store is a number, and the `input` store is a string. the `counter` store is incremented and decremented by buttons, and the `input` store is bound to an input field.

as the values of the stores change, the values are automatically synced across all instances of the namespace through the use of a websocket connection.

```svelte
<script lang="ts">
	import { Namespace } from "$lib/namespace";

    const ns = new Namespace("nathan", "soup");

    const counter = ns.writable("counter", 0);
    const inp = ns.writable("input", "hello");
</script>


<h1>{$counter}</h1>
<button on:click={() => counter.update((n) => n + 1)}>Increment</button>
<button on:click={() => counter.update((n) => n - 1)}>Decrement</button>

<hr/>

<h1>{$inp}</h1>
<input type="text" bind:value={$inp} />
```

**how it works**

the `Namespace` class creates a websocket connection to the server, and sends messages to the server when a store is updated. the server then broadcasts the new value to all connected clients.

each namespace has a `write_key` which is used to grant write access to create and write to stores within the namespace.

the server manages namespaces, which manage stores. the stores manage their own values and subscribers.

race conditions are very possible, and the server does not handle them.
