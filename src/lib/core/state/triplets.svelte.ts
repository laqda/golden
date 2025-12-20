export interface Triplet {
	letter1: number;
	letter2: number;
	letter3: number;
}

export function createTripletsState(rawTriplets: number[]) {
	const triplets: Triplet[] = [];
	for (let i = 0; i < rawTriplets.length; i += 3) {
		triplets.push({
			letter1: rawTriplets[i],
			letter2: rawTriplets[i + 1],
			letter3: rawTriplets[i + 2]
		});
	}

	let currentIndex = $state(0);

	return {
		get next() {
			return triplets[currentIndex];
		},
		get future() {
			return triplets.slice(currentIndex + 1);
		},
		get numberOfRemainingLetters() {
			return (triplets.length - currentIndex) * 3;
		},
		get currentIndex() {
			return currentIndex;
		},
		get total() {
			return triplets.length;
		},
		updateIndex(newIndex: number) {
			currentIndex = newIndex;
		}
	};
}

export type TripletsState = ReturnType<typeof createTripletsState>;
