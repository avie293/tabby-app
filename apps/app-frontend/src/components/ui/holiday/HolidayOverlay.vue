<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import { type Holiday, useHolidayTheme } from '@/composables/use-holiday-theme'

/**
 * Seasonal decorations drawn above the whole app: falling particles on a
 * canvas plus a few fixed ornaments. The overlay never takes clicks, pauses
 * while the window is hidden and drops the animation when the system asks
 * for reduced motion.
 */

const { active } = useHolidayTheme()

const canvas = ref<HTMLCanvasElement>()
const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)')

type Particle = {
	x: number
	y: number
	size: number
	speed: number
	drift: number
	phase: number
	spin: number
	angle: number
	color: string
	alpha: number
}

const PARTICLE_COUNT: Record<Holiday, number> = { christmas: 80, halloween: 7, easter: 45 }
const EASTER_COLORS = ['#f9a8d4', '#fde68a', '#a7f3d0', '#bfdbfe', '#ddd6fe']

let particles: Particle[] = []
let frame = 0
let lastTime = 0

function random(min: number, max: number) {
	return min + Math.random() * (max - min)
}

function createParticle(holiday: Holiday, width: number, height: number, anywhere: boolean) {
	const particle: Particle = {
		x: random(0, width),
		y: anywhere ? random(0, height) : random(-40, -10),
		size: 0,
		speed: 0,
		drift: 0,
		phase: random(0, Math.PI * 2),
		spin: 0,
		angle: random(0, Math.PI * 2),
		color: '#ffffff',
		alpha: 1,
	}
	if (holiday === 'christmas') {
		particle.size = random(1.2, 3.6)
		particle.speed = random(18, 55)
		particle.drift = random(8, 26)
		particle.alpha = random(0.45, 0.9)
	} else if (holiday === 'easter') {
		particle.size = random(4, 8)
		particle.speed = random(20, 45)
		particle.drift = random(20, 50)
		particle.spin = random(-1.6, 1.6)
		particle.color = EASTER_COLORS[Math.floor(Math.random() * EASTER_COLORS.length)]
		particle.alpha = random(0.55, 0.85)
	} else {
		// Bats fly across the window instead of falling.
		particle.x = anywhere ? random(0, width) : -40
		particle.y = random(height * 0.08, height * 0.6)
		particle.size = random(10, 18)
		particle.speed = random(60, 120)
		particle.drift = random(10, 30)
		particle.color = '#120a1c'
		particle.alpha = random(0.55, 0.85)
	}
	return particle
}

function drawBat(context: CanvasRenderingContext2D, particle: Particle, time: number) {
	const flap = Math.sin(time * 14 + particle.phase)
	const s = particle.size
	context.save()
	context.translate(particle.x, particle.y)
	context.fillStyle = particle.color
	context.globalAlpha = particle.alpha
	context.beginPath()
	context.moveTo(0, 0)
	context.quadraticCurveTo(-s * 0.6, -s * 0.8 * flap, -s * 1.6, -s * 0.4 * flap)
	context.quadraticCurveTo(-s * 1.1, s * 0.2, -s * 0.9, s * 0.15)
	context.quadraticCurveTo(-s * 0.6, s * 0.1, -s * 0.3, s * 0.3)
	context.lineTo(0, s * 0.15)
	context.lineTo(s * 0.3, s * 0.3)
	context.quadraticCurveTo(s * 0.6, s * 0.1, s * 0.9, s * 0.15)
	context.quadraticCurveTo(s * 1.1, s * 0.2, s * 1.6, -s * 0.4 * flap)
	context.quadraticCurveTo(s * 0.6, -s * 0.8 * flap, 0, 0)
	context.fill()
	context.beginPath()
	context.ellipse(0, s * 0.05, s * 0.22, s * 0.32, 0, 0, Math.PI * 2)
	context.fill()
	context.restore()
}

function step(time: number) {
	const element = canvas.value
	const holiday = active.value
	if (!element || !holiday) return
	const context = element.getContext('2d')
	if (!context) return
	const seconds = time / 1000
	const delta = Math.min((time - (lastTime || time)) / 1000, 0.05)
	lastTime = time
	const width = element.clientWidth
	const height = element.clientHeight
	context.clearRect(0, 0, width, height)

	for (const [index, particle] of particles.entries()) {
		if (holiday === 'halloween') {
			particle.x += particle.speed * delta
			particle.y += Math.sin(seconds * 1.5 + particle.phase) * particle.drift * delta
			if (particle.x > width + 40) particles[index] = createParticle(holiday, width, height, false)
			drawBat(context, particle, seconds)
			continue
		}
		particle.y += particle.speed * delta
		particle.x += Math.sin(seconds * 0.8 + particle.phase) * particle.drift * delta
		particle.angle += particle.spin * delta
		if (particle.y > height + 20) particles[index] = createParticle(holiday, width, height, false)
		context.globalAlpha = particle.alpha
		context.fillStyle = particle.color
		context.beginPath()
		if (holiday === 'easter') {
			context.ellipse(
				particle.x,
				particle.y,
				particle.size,
				particle.size * 0.55,
				particle.angle,
				0,
				Math.PI * 2,
			)
		} else {
			context.arc(particle.x, particle.y, particle.size, 0, Math.PI * 2)
		}
		context.fill()
	}
	context.globalAlpha = 1
	frame = requestAnimationFrame(step)
}

function resize() {
	const element = canvas.value
	if (!element) return
	const ratio = window.devicePixelRatio || 1
	element.width = element.clientWidth * ratio
	element.height = element.clientHeight * ratio
	element.getContext('2d')?.setTransform(ratio, 0, 0, ratio, 0, 0)
}

function stop() {
	cancelAnimationFrame(frame)
	frame = 0
	lastTime = 0
}

function start() {
	stop()
	const element = canvas.value
	const holiday = active.value
	if (!element || !holiday || reducedMotion.matches || document.hidden) return
	resize()
	particles = Array.from({ length: PARTICLE_COUNT[holiday] }, () =>
		createParticle(holiday, element.clientWidth, element.clientHeight, true),
	)
	frame = requestAnimationFrame(step)
}

function onVisibilityChange() {
	if (document.hidden) stop()
	else start()
}

watch([active, canvas], start, { flush: 'post' })
window.addEventListener('resize', resize)
document.addEventListener('visibilitychange', onVisibilityChange)
reducedMotion.addEventListener('change', start)
onBeforeUnmount(() => {
	stop()
	window.removeEventListener('resize', resize)
	document.removeEventListener('visibilitychange', onVisibilityChange)
	reducedMotion.removeEventListener('change', start)
})

const bulbColors = ['#ef4444', '#facc15', '#22c55e', '#3b82f6', '#f472b6']
const bulbs = computed(() => Array.from({ length: 40 }, (_, index) => index))
</script>

<template>
	<div v-if="active" class="holiday-overlay" aria-hidden="true">
		<canvas ref="canvas" class="absolute inset-0 h-full w-full" />

		<template v-if="active === 'christmas'">
			<svg class="lights" viewBox="0 0 1000 40" preserveAspectRatio="none">
				<path
					d="M0 6 Q12.5 22 25 6 T50 6 T75 6 T100 6 T125 6 T150 6 T175 6 T200 6 T225 6 T250 6 T275 6 T300 6 T325 6 T350 6 T375 6 T400 6 T425 6 T450 6 T475 6 T500 6 T525 6 T550 6 T575 6 T600 6 T625 6 T650 6 T675 6 T700 6 T725 6 T750 6 T775 6 T800 6 T825 6 T850 6 T875 6 T900 6 T925 6 T950 6 T975 6 T1000 6"
					fill="none"
					stroke="#2d3a2e"
					stroke-width="1.6"
					vector-effect="non-scaling-stroke"
				/>
			</svg>
			<div class="bulbs">
				<span
					v-for="index in bulbs"
					:key="index"
					class="bulb"
					:style="{
						left: `${(index + 0.5) * 2.5}%`,
						background: bulbColors[index % bulbColors.length],
						color: bulbColors[index % bulbColors.length],
						animationDelay: `${(index % 7) * 0.35}s`,
					}"
				/>
			</div>
			<div class="snowdrift" />
		</template>

		<template v-else-if="active === 'halloween'">
			<svg class="web web-left" viewBox="0 0 160 160">
				<g fill="none" stroke="#ffffff" stroke-opacity="0.28" stroke-width="1.2">
					<path d="M0 0 L160 40 M0 0 L120 120 M0 0 L40 160 M0 0 L90 150 M0 0 L150 85" />
					<path
						d="M28 7 Q32 22 21 28 Q14 34 7 28 M56 14 Q63 42 42 56 Q28 68 14 56 M84 21 Q95 63 63 84 Q42 102 21 84 M112 28 Q126 84 84 112 Q56 136 28 112"
					/>
				</g>
			</svg>
			<svg class="web web-right" viewBox="0 0 160 160">
				<g fill="none" stroke="#ffffff" stroke-opacity="0.28" stroke-width="1.2">
					<path d="M0 0 L160 40 M0 0 L120 120 M0 0 L40 160 M0 0 L90 150 M0 0 L150 85" />
					<path
						d="M28 7 Q32 22 21 28 Q14 34 7 28 M56 14 Q63 42 42 56 Q28 68 14 56 M84 21 Q95 63 63 84 Q42 102 21 84 M112 28 Q126 84 84 112 Q56 136 28 112"
					/>
				</g>
			</svg>
			<div class="pumpkin-glow" />
			<svg class="pumpkin" viewBox="0 0 120 100">
				<path
					d="M58 18 Q60 4 70 2"
					stroke="#4d7c0f"
					stroke-width="6"
					fill="none"
					stroke-linecap="round"
				/>
				<ellipse cx="38" cy="58" rx="30" ry="36" fill="#ea580c" />
				<ellipse cx="82" cy="58" rx="30" ry="36" fill="#ea580c" />
				<ellipse cx="60" cy="58" rx="30" ry="38" fill="#f97316" />
				<path d="M40 50 L50 40 L56 52 Z M80 50 L70 40 L64 52 Z" fill="#fde047" />
				<path d="M36 70 Q60 92 84 70 L76 74 L70 68 L64 76 L56 68 L50 76 L44 68 Z" fill="#fde047" />
			</svg>
		</template>

		<template v-else-if="active === 'easter'">
			<svg class="eggs" viewBox="0 0 220 110">
				<g>
					<ellipse cx="50" cy="70" rx="34" ry="44" fill="#f9a8d4" />
					<path
						d="M18 62 L30 54 L42 62 L54 54 L66 62 L78 54 L84 60"
						stroke="#ffffff"
						stroke-width="5"
						fill="none"
					/>
					<circle cx="40" cy="84" r="5" fill="#fde68a" />
					<circle cx="62" cy="88" r="5" fill="#a7f3d0" />
				</g>
				<g>
					<ellipse cx="118" cy="62" rx="38" ry="50" fill="#bfdbfe" />
					<path d="M82 50 Q118 64 154 50" stroke="#ddd6fe" stroke-width="8" fill="none" />
					<path d="M84 76 Q118 90 152 76" stroke="#fde68a" stroke-width="8" fill="none" />
				</g>
				<g>
					<ellipse cx="184" cy="76" rx="30" ry="40" fill="#a7f3d0" />
					<circle cx="174" cy="64" r="5" fill="#ffffff" />
					<circle cx="192" cy="72" r="5" fill="#ffffff" />
					<circle cx="180" cy="90" r="5" fill="#ffffff" />
				</g>
			</svg>
		</template>
	</div>
</template>

<style scoped>
.holiday-overlay {
	position: fixed;
	inset: 0;
	z-index: 9999;
	pointer-events: none;
	overflow: hidden;
}

.lights {
	position: absolute;
	top: 0;
	left: 0;
	width: 100%;
	height: 26px;
}

.bulbs {
	position: absolute;
	top: 0;
	left: 0;
	width: 100%;
	height: 26px;
}

.bulb {
	position: absolute;
	top: 10px;
	width: 7px;
	height: 11px;
	margin-left: -3.5px;
	border-radius: 50% 50% 50% 50% / 40% 40% 60% 60%;
	box-shadow: 0 0 8px 2px currentColor;
	animation: twinkle 2.4s ease-in-out infinite;
}

@keyframes twinkle {
	0%,
	100% {
		opacity: 1;
	}
	50% {
		opacity: 0.35;
		box-shadow: 0 0 2px 0 currentColor;
	}
}

.snowdrift {
	position: absolute;
	left: 0;
	right: 0;
	bottom: 0;
	height: 14px;
	background: radial-gradient(ellipse at 50% 100%, rgb(255 255 255 / 0.22), transparent 70%);
}

.web {
	position: absolute;
	top: 0;
	width: 150px;
	height: 150px;
}

.web-left {
	left: 0;
}

.web-right {
	right: 0;
	transform: scaleX(-1);
}

.pumpkin-glow {
	position: absolute;
	right: -60px;
	bottom: -60px;
	width: 260px;
	height: 260px;
	background: radial-gradient(circle, rgb(249 115 22 / 0.28), transparent 65%);
	animation: flicker 3s ease-in-out infinite;
}

.pumpkin {
	position: absolute;
	right: 18px;
	bottom: 10px;
	width: 70px;
	height: 58px;
	filter: drop-shadow(0 0 10px rgb(249 115 22 / 0.6));
}

@keyframes flicker {
	0%,
	100% {
		opacity: 1;
	}
	40% {
		opacity: 0.7;
	}
	60% {
		opacity: 0.9;
	}
}

.eggs {
	position: absolute;
	right: 16px;
	bottom: -18px;
	width: 150px;
	height: 76px;
	filter: drop-shadow(0 4px 8px rgb(0 0 0 / 0.3));
}

@media (prefers-reduced-motion: reduce) {
	.bulb,
	.pumpkin-glow {
		animation: none;
	}
}
</style>
