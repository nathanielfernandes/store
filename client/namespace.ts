import { writable } from "svelte/store";
import { browser } from "$app/environment";

import type { ClientMessage, ClientMessageMap, ClientMessageTypes, ServerMessage } from "./messages";

export class Namespace {
    public name: string;
    public write_key: string | null = null;
    public stringify: boolean = true;

    private ws: WebSocket | null = null;
    private handlers: Map<string, (value: any) => void>;
    private initial: Map<string, any> = new Map();

    private ready = false;

    constructor(name: string, write_key: string | null = null, stringify: boolean = true) {
        this.name = name;
        this.write_key = write_key;
        this.stringify = stringify;

        this.handlers = new Map();

        // check if browser
        if (browser) {
            this.connect();
        }
    }

    private connect() {
        const write = this.write_key ? `/${this.write_key}` : '';
        this.ws = new WebSocket(`ws://wsl:3000/ws/${this.name}${write}`);

        this.hook_ws();
    }

    private hook_ws() {
        if (!this.ws) {
            console.error('no websocket connection');
            return;
        }

        this.ws.onopen = this.onopen.bind(this);
        this.ws.onclose = this.onclose.bind(this);
        this.ws.onmessage = this.onmessage.bind(this);
    }

    private stringifix(value: any): string {
        if (this.stringify) {
            return JSON.stringify(value);
        }

        // if not stringifying make sure it's a string
        if (typeof value !== 'string') {
            return value.toString();
        }

        return value;
    }

    private onopen() {
        this.ready = true;
        this.handlers.forEach((_, store_name) => {
            const initial = this.initial.get(store_name);
            this.send_message('Subscribe', { store: store_name, initial: this.stringifix(initial)});
        });
    }

    private onclose() {
        this.ready = false;
        // try to reconnect
        setTimeout(() => {
            this.connect();
        }, 1000);
    }

    private onmessage(event: MessageEvent) {
        try {
            const msg: ServerMessage = JSON.parse(event.data);
            switch (msg.type) {
                case 'Update':
                    const value = this.stringify ? JSON.parse(msg.value) : msg.value;
                    this.handlers.get(msg.store)?.(value);
                    break;
                default:
                    console.error('unknown message type');
            }
        } catch (error) {
            console.error(error);
        }
    }

    private send_message<T extends ClientMessageTypes>(type: T, value: ClientMessageMap<T>) {
        if (!this.ws) {
            console.error('no websocket connection');
            return;
        }

        const message: ClientMessage = { type, ...value } as any;
        const serialized = JSON.stringify(message);

        this.ws.send(serialized);
    }

    public set(store_name: string, value: any) {
        this.send_message('Set', { store: store_name, value: this.stringifix(value)});
    }

    public get(store_name: string) {
        this.send_message('Get', { store: store_name });
    }

    private subscribe(store_name: string, initial: any, handler: (value: any) => void) {
        this.handlers.set(store_name, handler);
        this.initial.set(store_name, initial);
        
        if (!this.ready) {
            return;
        }
        this.send_message('Subscribe', { store: store_name, initial: this.stringifix(initial)});
    }
    
    // create a new readable store
    public readable<T>(store_name: string, initial: T) {
        const store = writable(initial);
        const handle = (value: T) => {
            store.set(value);
        }   
        this.subscribe(store_name, initial, handle);

        return { subscribe: store.subscribe };
    }

    // create a new writable store
    public writable<T>(store_name: string, initial: T) {
        const store = writable(initial);
        const handle = (value: T) => {
            store.set(value);
        }   
        this.subscribe(store_name, initial, handle);

        return {
            subscribe: store.subscribe,
            set: (value: T) => {
                if (!this.ready) { return; }
                this.set(store_name, value);
            },
            update: (fn: (value: T) => T) => {
                store.update((current) => {
                    const value = fn(current);
                    if (!this.ready) { return current }
                    this.set(store_name, value);
                    return current;
                });
            },
            unsubscribe: () => {
                this.send_message('Unsubscribe', { store: store_name });
                this.handlers.delete(store_name);
                this.initial.delete(store_name);
            },
            changeto: (store_name: string, initial: T) => {
                this.send_message('Unsubscribe', { store: store_name });
                this.handlers.delete(store_name);
                this.initial.delete(store_name);
                this.subscribe(store_name, initial, handle);
            }
        }
    }
}