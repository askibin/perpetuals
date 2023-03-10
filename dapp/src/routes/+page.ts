import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

/**
 * Throw a redirect to /long
 */
export const load = (({ params }) => {
	throw redirect(302, '/long');
}) satisfies PageLoad;
