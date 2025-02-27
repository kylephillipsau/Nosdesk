<!-- ErrorView.vue -->
<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router';
import { onMounted, onUnmounted, ref } from 'vue';

type SpikeDirection = 'vertical' | 'horizontal' | null;

const route = useRoute();
const router = useRouter();

const goBack = () => router.back();
const goHome = () => router.push('/');

// Animation setup
const redOffset = ref<SVGFEOffsetElement | null>(null);
const greenOffset = ref<SVGFEOffsetElement | null>(null);
const blueOffset = ref<SVGFEOffsetElement | null>(null);
const redDisplacement = ref<SVGFEDisplacementMapElement | null>(null);
const greenDisplacement = ref<SVGFEDisplacementMapElement | null>(null);
const blueDisplacement = ref<SVGFEDisplacementMapElement | null>(null);
const frameId = ref<number>(0);
const redTurbulence = ref<SVGFETurbulenceElement | null>(null);
const greenTurbulence = ref<SVGFETurbulenceElement | null>(null);
const blueTurbulence = ref<SVGFETurbulenceElement | null>(null);

let glitchCounterRed = 0;
let glitchCounterGreen = 0;
let glitchCounterBlue = 0;
let spikeDirectionRed: SpikeDirection = null;
let spikeDirectionGreen: SpikeDirection = null;
let spikeDirectionBlue: SpikeDirection = null;
let wobbleTime = 0;

const animate = () => {
  const innerAnimate = () => {
    wobbleTime += 0.05;

    // Default baseFrequency for normal warbling
    const defaultFreq = '0.01';

    // Calculate subtle position drift using sine/cosine
    const driftAmplitude = 5; // Small amplitude for subtle movement
    const redDriftX = Math.sin(wobbleTime * 0.3) * driftAmplitude;
    const redDriftY = Math.cos(wobbleTime * 0.2) * driftAmplitude;
    const greenDriftX = Math.sin(wobbleTime * 0.25 + 1) * driftAmplitude;
    const greenDriftY = Math.cos(wobbleTime * 0.35 + 2) * driftAmplitude;
    const blueDriftX = Math.sin(wobbleTime * 0.4 + 3) * driftAmplitude;
    const blueDriftY = Math.cos(wobbleTime * 0.15 + 4) * driftAmplitude;

    // Check for glitch triggers
    if (Math.random() < 0.0056 && glitchCounterRed === 0) {
      glitchCounterRed = 5 + Math.floor(Math.random() * 20);
      spikeDirectionRed = Math.random() < 0.5 ? 'vertical' : 'horizontal';
    }
    if (Math.random() < 0.0056 && glitchCounterGreen === 0) {
      glitchCounterGreen = 5 + Math.floor(Math.random() * 20);
      spikeDirectionGreen = Math.random() < 0.5 ? 'vertical' : 'horizontal';
    }
    if (Math.random() < 0.0056 && glitchCounterBlue === 0) {
      glitchCounterBlue = 5 + Math.floor(Math.random() * 20);
      spikeDirectionBlue = Math.random() < 0.5 ? 'vertical' : 'horizontal';
    }

    // Base scale and smooth oscillation
    const baseScale = 15;
    const amplitude = 10;
    const scaleRed = baseScale + Math.sin(wobbleTime + 0) * amplitude;
    const scaleGreen = baseScale + Math.sin(wobbleTime + 2) * amplitude;
    const scaleBlue = baseScale + Math.sin(wobbleTime + 4) * amplitude;

    // Red Channel
    if (glitchCounterRed > 0) {
      const freqX = spikeDirectionRed === 'vertical' ? '0.1' : '0.001';
      const freqY = spikeDirectionRed === 'vertical' ? '0.001' : '0.1';
      redTurbulence.value?.setAttributeNS(null, 'baseFrequency', `${freqX} ${freqY}`);
      glitchCounterRed--;
    } else {
      redTurbulence.value?.setAttributeNS(null, 'baseFrequency', defaultFreq);
    }
    redDisplacement.value?.setAttributeNS(null, 'scale', `${scaleRed}`);
    redOffset.value?.setAttributeNS(null, 'dx', `${redDriftX}`);
    redOffset.value?.setAttributeNS(null, 'dy', `${redDriftY}`);

    // Green Channel
    if (glitchCounterGreen > 0) {
      const freqX = spikeDirectionGreen === 'vertical' ? '0.1' : '0.001';
      const freqY = spikeDirectionGreen === 'vertical' ? '0.001' : '0.1';
      greenTurbulence.value?.setAttributeNS(null, 'baseFrequency', `${freqX} ${freqY}`);
      glitchCounterGreen--;
    } else {
      greenTurbulence.value?.setAttributeNS(null, 'baseFrequency', defaultFreq);
    }
    greenDisplacement.value?.setAttributeNS(null, 'scale', `${scaleGreen}`);
    greenOffset.value?.setAttributeNS(null, 'dx', `${greenDriftX}`);
    greenOffset.value?.setAttributeNS(null, 'dy', `${greenDriftY}`);

    // Blue Channel
    if (glitchCounterBlue > 0) {
      const freqX = spikeDirectionBlue === 'vertical' ? '0.1' : '0.001';
      const freqY = spikeDirectionBlue === 'vertical' ? '0.001' : '0.1';
      blueTurbulence.value?.setAttributeNS(null, 'baseFrequency', `${freqX} ${freqY}`);
      glitchCounterBlue--;
    } else {
      blueTurbulence.value?.setAttributeNS(null, 'baseFrequency', defaultFreq);
    }
    blueDisplacement.value?.setAttributeNS(null, 'scale', `${scaleBlue}`);
    blueOffset.value?.setAttributeNS(null, 'dx', `${blueDriftX}`);
    blueOffset.value?.setAttributeNS(null, 'dy', `${blueDriftY}`);

    // Schedule next frame
    frameId.value = requestAnimationFrame(innerAnimate);
  };
  innerAnimate();
};

onMounted(() => {
  animate();
});

onUnmounted(() => {
  if (frameId.value) {
    cancelAnimationFrame(frameId.value);
  }
});
</script>

<template>
  <!-- Template remains unchanged -->
  <div class="min-h-screen w-full flex items-center justify-center bg-slate-900 p-4">
    <div class="flex flex-col gap-2 text-center">
      <svg ref="svg" class="error-svg">
        <defs>
          <filter id="rgbGlitch">
            <!-- Red Channel -->
            <feTurbulence ref="redTurbulence" type="turbulence" baseFrequency="0.01" numOctaves="3" result="redTurb" />
            <feOffset ref="redOffset" in="redTurb" dx="0" result="redOffsetTurb" />
            <feDisplacementMap ref="redDisplacement" in="SourceGraphic" in2="redOffsetTurb" scale="15" xChannelSelector="R" yChannelSelector="G" result="redDistorted" />
            <feColorMatrix in="redDistorted" type="matrix" values="1 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0" result="red" />

            <!-- Green Channel -->
            <feTurbulence ref="greenTurbulence" type="turbulence" baseFrequency="0.01" numOctaves="3" result="greenTurb" />
            <feOffset ref="greenOffset" in="greenTurb" dx="0" result="greenOffsetTurb" />
            <feDisplacementMap ref="greenDisplacement" in="SourceGraphic" in2="greenOffsetTurb" scale="15" xChannelSelector="R" yChannelSelector="G" result="greenDistorted" />
            <feColorMatrix in="greenDistorted" type="matrix" values="0 0 0 0 0  0 1 0 0 0  0 0 0 0 0  0 0 0 1 0" result="green" />

            <!-- Blue Channel -->
            <feTurbulence ref="blueTurbulence" type="turbulence" baseFrequency="0.01" numOctaves="3" result="blueTurb" />
            <feOffset ref="blueOffset" in="blueTurb" dx="0" result="blueOffsetTurb" />
            <feDisplacementMap ref="blueDisplacement" in="SourceGraphic" in2="blueOffsetTurb" scale="15" xChannelSelector="R" yChannelSelector="G" result="blueDistorted" />
            <feColorMatrix in="blueDistorted" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 1 0 0  0 0 0 1 0" result="blue" />

            <!-- Additive Blending with feBlend -->
            <feBlend in="red" in2="green" mode="screen" result="redGreen" />
            <feBlend in="redGreen" in2="blue" mode="screen" result="final" />
          </filter>
        </defs>
        <text x="50%" y="50%" text-anchor="middle" dominant-baseline="middle" filter="url(#rgbGlitch)" class="error-text">{{ route.params.code || '404' }}</text>
      </svg>

      <div class="mt-4 text-xl text-slate-300">
        {{ route.params.message || 'Page not found' }}
      </div>

      <p class="mt-2 text-slate-400">
        The page you're looking for doesn't exist or you may not have access to it.
      </p>

      <div class="mt-8 flex gap-4 justify-center">
        <button @click="goBack" class="px-4 py-2 text-sm font-medium text-slate-300 hover:text-white transition-colors">
          ← Go back
        </button>
        <button @click="goHome" class="px-4 py-2 text-sm font-medium bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
          Go to Dashboard
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.error-svg {
  width: 46rem;
  height: 18rem;
}

.error-text {
  font-size: 14rem;
  fill: white;
}

@keyframes float {
  0%, 100% { transform: translateY(0px); }
  50% { transform: translateY(-10px); }
}

.error-svg {
  animation: float 6s ease-in-out infinite;
}
</style>