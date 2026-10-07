// Renders the news thumbnails in news/<slug>/thumbnail.webp.
//
// Every thumbnail has the same look: a dark purple background of isometric
// blocks with frosted glass shapes in front. Add a new article by adding a
// drawing function to `thumbnails` below, then run
// `node news/render-thumbnails.mjs` from the repository root.

import { mkdirSync, writeFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = dirname(fileURLToPath(import.meta.url))
const require = createRequire(join(root, '..', 'apps', 'app-frontend', 'package.json'))
const sharp = require(
	require.resolve('sharp', {
		paths: [join(root, '..', 'node_modules', '.pnpm', 'node_modules')],
	}),
)

const WIDTH = 1920
const HEIGHT = 1080
const BRAND = ['#d4bcff', '#9d6bf5']

/** Seeded random numbers, so every render of a thumbnail looks the same. */
function random(seed) {
	let value = seed
	return () => {
		value = (value * 1664525 + 1013904223) % 4294967296
		return value / 4294967296
	}
}

/** Columns of isometric blocks with random heights, drawn back to front. */
function blocks(seed) {
	const next = random(seed)
	const size = 230
	const dx = size * Math.cos(Math.PI / 6)
	const dy = size * Math.sin(Math.PI / 6)
	const shapes = []
	for (let row = -4; row < 9; row++) {
		for (let column = -4; column < 9; column++) {
			const height = size * (0.4 + next() * 2.2)
			const x = (column - row) * dx + WIDTH / 2
			const y = (column + row) * dy - 260
			const shade = 0.75 + next() * 0.4
			const color = (light) => {
				const base = [44, 30, 66].map((channel) => Math.round(channel * light * shade))
				return `rgb(${base.join(',')})`
			}
			const top = [
				[x, y - height],
				[x + dx, y + dy - height],
				[x, y + 2 * dy - height],
				[x - dx, y + dy - height],
			]
			const left = [
				[x - dx, y + dy - height],
				[x, y + 2 * dy - height],
				[x, y + 2 * dy],
				[x - dx, y + dy],
			]
			const right = [
				[x + dx, y + dy - height],
				[x, y + 2 * dy - height],
				[x, y + 2 * dy],
				[x + dx, y + dy],
			]
			const polygon = (points, fill) =>
				`<polygon points="${points.map((point) => point.join(',')).join(' ')}" fill="${fill}"/>`
			shapes.push({
				order: row + column,
				svg: polygon(left, color(0.85)) + polygon(right, color(0.7)) + polygon(top, color(1.08)),
			})
		}
	}
	return shapes
		.sort((a, b) => a.order - b.order)
		.map((shape) => shape.svg)
		.join('')
}

const defs = `
	<defs>
		<radialGradient id="vignette" cx="0.5" cy="0.45" r="0.75">
			<stop offset="0.45" stop-color="#0d0a14" stop-opacity="0"/>
			<stop offset="1" stop-color="#0d0a14" stop-opacity="0.75"/>
		</radialGradient>
		<linearGradient id="brand" x1="0" y1="0" x2="1" y2="1">
			<stop offset="0" stop-color="${BRAND[0]}"/>
			<stop offset="1" stop-color="${BRAND[1]}"/>
		</linearGradient>
		<linearGradient id="brand-bar" x1="0" y1="0" x2="1" y2="0">
			<stop offset="0" stop-color="#c7a6ff"/>
			<stop offset="1" stop-color="#9d6bf5"/>
		</linearGradient>
		<linearGradient id="teal" x1="0" y1="0" x2="0" y2="1">
			<stop offset="0" stop-color="#4fd1a5"/>
			<stop offset="1" stop-color="#2a9d7f"/>
		</linearGradient>
		<linearGradient id="orange" x1="0" y1="0" x2="0" y2="1">
			<stop offset="0" stop-color="#f29a5a"/>
			<stop offset="1" stop-color="#d06a3a"/>
		</linearGradient>
		<linearGradient id="blue" x1="0" y1="0" x2="0" y2="1">
			<stop offset="0" stop-color="#4c84d6"/>
			<stop offset="1" stop-color="#5aa0c8"/>
		</linearGradient>
		<linearGradient id="pink" x1="0" y1="0" x2="0" y2="1">
			<stop offset="0" stop-color="#c25aa8"/>
			<stop offset="1" stop-color="#b86ab8"/>
		</linearGradient>
		<filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
			<feDropShadow dx="0" dy="18" stdDeviation="26" flood-color="#000" flood-opacity="0.35"/>
		</filter>
	</defs>`

/** A frosted glass panel. */
function glass(x, y, width, height, radius, extra = '') {
	return `<rect x="${x}" y="${y}" width="${width}" height="${height}" rx="${radius}"
		fill="#ffffff" fill-opacity="0.08" stroke="#ffffff" stroke-opacity="0.16" stroke-width="3"
		filter="url(#shadow)" ${extra}/>`
}

/** A grey placeholder line, like text in the reference thumbnails. */
function line(x, y, width, height = 34) {
	return `<rect x="${x}" y="${y}" width="${width}" height="${height}" rx="${height / 2}" fill="#ffffff" fill-opacity="0.22"/>`
}

function paw(cx, cy, scale) {
	return `<g transform="translate(${cx} ${cy}) scale(${scale}) translate(-12 -12.4)" fill="#ffffff" stroke="#ffffff" stroke-width="1.4" stroke-linejoin="round">
		<circle cx="11" cy="4" r="2.3"/><circle cx="18" cy="8" r="2.3"/><circle cx="20" cy="16" r="2.3"/>
		<path d="M9 10a5 5 0 0 1 5 5v3.5a3.5 3.5 0 0 1-6.84 1.045Q6.52 17.48 4.46 16.84A3.5 3.5 0 0 1 5.5 10Z"/>
	</g>`
}

/** A round badge with a brand colored glass look, like the link icon in the reference. */
function badge(cx, cy, r, content) {
	return `<circle cx="${cx}" cy="${cy}" r="${r}" fill="#3b2a5c" fill-opacity="0.92" stroke="#ffffff" stroke-opacity="0.18" stroke-width="3" filter="url(#shadow)"/>${content}`
}

function slider(x, y, width, value) {
	const knob = x + width * value
	return `<rect x="${x}" y="${y}" width="${width}" height="54" rx="27" fill="#ffffff" fill-opacity="0.22" stroke="#ffffff" stroke-opacity="0.2" stroke-width="2"/>
		<rect x="${x}" y="${y}" width="${knob - x}" height="54" rx="27" fill="url(#brand-bar)"/>
		<rect x="${knob - 20}" y="${y - 30}" width="40" height="114" rx="20" fill="#c7a6ff" stroke="#ffffff" stroke-opacity="0.5" stroke-width="2"/>`
}

const check = (cx, cy, scale) =>
	`<path d="M${cx - 34 * scale} ${cy} l${24 * scale} ${24 * scale} l${46 * scale} -${50 * scale}" fill="none" stroke="#ffffff" stroke-width="${16 * scale}" stroke-linecap="round" stroke-linejoin="round"/>`

const arrow = (cx, cy, scale) =>
	`<path d="M${cx - 50 * scale} ${cy} h${96 * scale} m-${40 * scale} -${42 * scale} l${42 * scale} ${42 * scale} l-${42 * scale} ${42 * scale}" fill="none" stroke="#ffffff" stroke-width="${18 * scale}" stroke-linecap="round" stroke-linejoin="round"/>`

/** A file card with a folded corner, a colored icon and a label. */
function file(x, y, fill, label, icon = '') {
	const width = 380
	const height = 470
	return `<path d="M${x + 36} ${y} h${width - 126} l90 90 v${height - 126} a36 36 0 0 1 -36 36 h-${width - 72} a36 36 0 0 1 -36 -36 v-${height - 72} a36 36 0 0 1 36 -36 Z"
			fill="#ffffff" fill-opacity="0.08" stroke="#ffffff" stroke-opacity="0.18" stroke-width="3" filter="url(#shadow)"/>
		<path d="M${x + width - 90} ${y} v54 a36 36 0 0 0 36 36 h54" fill="#ffffff" fill-opacity="0.12" stroke="#ffffff" stroke-opacity="0.18" stroke-width="3"/>
		<rect x="${x + 110}" y="${y + 120}" width="160" height="160" rx="40" fill="${fill}"/>
		${icon}
		<text x="${x + width / 2}" y="${y + 390}" text-anchor="middle" font-family="Segoe UI, Inter, sans-serif" font-weight="700" font-size="64" fill="#ffffff" fill-opacity="0.9">${label}</text>`
}

function windowFrame(x, y, width, height) {
	return `${glass(x, y, width, height, 48)}
		<circle cx="${x + 50}" cy="${y + 48}" r="16" fill="#ffffff" fill-opacity="0.2"/>
		<circle cx="${x + 96}" cy="${y + 48}" r="16" fill="#ffffff" fill-opacity="0.2"/>
		<circle cx="${x + 142}" cy="${y + 48}" r="16" fill="#ffffff" fill-opacity="0.2"/>`
}

const thumbnails = {
	welcome: () => `
		${slider(340, 400, 760, 0.62)}
		${slider(340, 624, 760, 0.28)}
		${badge(1410, 540, 176, paw(1410, 540, 9))}`,

	'modrinth-app-import': () => `
		${windowFrame(150, 220, 640, 640)}
		<rect x="210" y="320" width="150" height="150" rx="36" fill="url(#teal)"/>
		${line(390, 345, 300)}${line(390, 405, 200)}
		<rect x="210" y="510" width="150" height="150" rx="36" fill="url(#orange)"/>
		${line(390, 535, 260)}${line(390, 595, 170)}
		${line(210, 710, 520, 60)}
		${windowFrame(1130, 220, 640, 640)}
		<rect x="1190" y="320" width="150" height="150" rx="36" fill="url(#teal)"/>
		${line(1370, 345, 300)}${line(1370, 405, 200)}
		<rect x="1190" y="510" width="150" height="150" rx="36" fill="url(#orange)"/>
		${line(1370, 535, 260)}${line(1370, 595, 170)}
		<rect x="1190" y="710" width="520" height="60" rx="30" fill="url(#brand-bar)"/>
		${badge(960, 540, 120, arrow(960, 540, 1.1))}`,

	'modpack-export': () => `
		${file(250, 300, 'url(#teal)', '.mrpack')}
		${file(770, 300, 'url(#orange)', '.zip')}
		${file(1290, 300, 'url(#brand)', '.tabpack', paw(1480, 500, 4.8))}`,

	'more-stable-downloads': () => `
		${glass(330, 170, 1260, 740, 56)}
		${[0, 1, 2]
			.map((index) => {
				const y = 240 + index * 220
				const colors = ['url(#blue)', 'url(#pink)', 'url(#teal)']
				return `${glass(390, y, 1140, 180, 40)}
					<rect x="430" y="${y + 30}" width="120" height="120" rx="30" fill="${colors[index]}"/>
					${line(590, y + 42, 360)}
					<rect x="590" y="${y + 104}" width="760" height="30" rx="15" fill="#ffffff" fill-opacity="0.18"/>
					<rect x="590" y="${y + 104}" width="760" height="30" rx="15" fill="url(#brand-bar)"/>
					<circle cx="1440" cy="${y + 90}" r="46" fill="url(#brand)"/>
					${check(1440, y + 92, 0.62)}`
			})
			.join('')}`,
}

const seeds = {
	welcome: 7,
	'modrinth-app-import': 19,
	'modpack-export': 23,
	'more-stable-downloads': 31,
}

for (const [slug, draw] of Object.entries(thumbnails)) {
	const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${WIDTH}" height="${HEIGHT}" viewBox="0 0 ${WIDTH} ${HEIGHT}">
		${defs}
		<rect width="${WIDTH}" height="${HEIGHT}" fill="#1c1428"/>
		${blocks(seeds[slug] ?? 1)}
		<rect width="${WIDTH}" height="${HEIGHT}" fill="url(#vignette)"/>
		${draw()}
	</svg>`
	const directory = join(root, slug)
	mkdirSync(directory, { recursive: true })
	writeFileSync(
		join(directory, 'thumbnail.webp'),
		await sharp(Buffer.from(svg)).webp({ quality: 86 }).toBuffer(),
	)
	console.log(`Rendered ${slug}`)
}
