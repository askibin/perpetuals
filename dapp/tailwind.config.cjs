/** @type {import('tailwindcss').Config} */
module.exports = {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			fontFamily: {
				pixel: ['Pixel']
			},
			keyframes: {
				circlespin: {
					'0%': {
						backgroundPosition: '0 0'
					},

					'50%': {
						backgroundPosition: '400% 0'
					},
					'100%': {
						backgroundPosition: '0 0'
					}
				}
			}
		}
	},
	plugins: []
};
