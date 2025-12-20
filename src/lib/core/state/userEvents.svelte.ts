import type { Position } from '$lib/wasm/golden_core';
import { getContext, setContext } from 'svelte';

const USER_EVENTS_CONTEXT_KEY = Symbol('user_events');

export interface UserEventsContext {
	click: (position: Position) => void;
	hover: (position: Position) => void;
}

export function createUserEventsState() {
	let userClicks = $state<Position[]>([]);
	let lastHovered = $state<Position | null>(null);

	const userEvents: UserEventsContext = {
		click(position: Position) {
			userClicks = [...userClicks, position];
		},
		hover(position: Position) {
			lastHovered = position;
		}
	};

	setContext<UserEventsContext>(USER_EVENTS_CONTEXT_KEY, userEvents);

	return {
		get lastHovered() {
			return lastHovered;
		},
		extractUserClicks(): Position[] {
			const clicks = userClicks;
			userClicks = [];
			return clicks;
		},
		...userEvents
	};
}

export type UserEventsState = ReturnType<typeof createUserEventsState>;

export function useUserEvents(): UserEventsContext {
	return getContext<UserEventsContext>(USER_EVENTS_CONTEXT_KEY);
}
