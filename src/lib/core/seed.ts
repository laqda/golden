export function seedOfTheDay(): number {
	const now = new Date();
	const seed = now.getUTCFullYear() * 10000 + (now.getUTCMonth() + 1) * 100 + now.getUTCDate();
	return seed;
}
