<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import ProgressBar from '@/components/ticketComponents/ProgressBar.vue';

const props = defineProps<{
  src: string;
}>();

const videoRef = ref<HTMLVideoElement | null>(null);
const isPlaying = ref(false);
const currentTime = ref(0);
const duration = ref(0);
const volume = ref(1);
const isFullscreen = ref(false);
const showControls = ref(false);
const controlsTimeout = ref<number | null>(null);
const wasPlaying = ref(false);

// Format time for display
const formatTime = (seconds: number): string => {
  if (!seconds || isNaN(seconds)) return "0:00";
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
};

// Logging function for debugging
const log = (event: string, details?: any) => {
  console.log(`[VideoPlayer] ${event}`, details || '');
};

// Toggle play/pause
const togglePlay = (event?: Event) => {
  log('togglePlay', { event: event?.type });
  if (event) {
    event.preventDefault();
    event.stopPropagation();
  }
  if (!videoRef.value) return;
  if (isPlaying.value) {
    videoRef.value.pause();
    isPlaying.value = false;
  } else {
    videoRef.value.play();
    isPlaying.value = true;
  }
};

// Update volume
const updateVolume = (event: Event) => {
  const input = event.target as HTMLInputElement;
  volume.value = Number(input.value);
  if (videoRef.value) {
    videoRef.value.volume = volume.value;
  }
};

// Toggle fullscreen
const toggleFullscreen = async () => {
  if (!videoRef.value) return;
  if (!document.fullscreenElement) {
    await videoRef.value.requestFullscreen();
    isFullscreen.value = true;
  } else {
    await document.exitFullscreen();
    isFullscreen.value = false;
  }
};

// Handle seek event from ProgressBar
const handleSeek = (time: number) => {
  log('handleSeek', { time, currentTime: videoRef.value?.currentTime });
  if (videoRef.value) {
    videoRef.value.currentTime = time;
    currentTime.value = time;
  }
};

// Handle drag start: Pause video and disable pointer events on video
const handleDragStart = () => {
  log('handleDragStart', { wasPlaying: isPlaying.value });
  if (videoRef.value) {
    wasPlaying.value = isPlaying.value;
    if (isPlaying.value) {
      videoRef.value.pause();
      isPlaying.value = false;
    }
    videoRef.value.style.pointerEvents = 'none'; // Disable video interaction
  }
};

// Handle drag end: Resume playback and re-enable pointer events on video
const handleDragEnd = () => {
  log('handleDragEnd', { wasPlaying: wasPlaying.value });
  if (videoRef.value) {
    videoRef.value.style.pointerEvents = 'auto'; // Re-enable video interaction
    if (wasPlaying.value) {
      videoRef.value.play();
      isPlaying.value = true;
    }
  }
};

// Show controls temporarily
const showControlsTemporarily = () => {
  log('showControls');
  showControls.value = true;
  if (controlsTimeout.value) {
    clearTimeout(controlsTimeout.value);
  }
  controlsTimeout.value = window.setTimeout(() => {
    if (isPlaying.value) {
      showControls.value = false;
    }
  }, 2000);
};

// Setup video event listeners
onMounted(() => {
  log('mounted');
  if (!videoRef.value) return;
  videoRef.value.addEventListener('play', () => {
    log('video:play');
    isPlaying.value = true;
  });
  videoRef.value.addEventListener('pause', () => {
    log('video:pause');
    isPlaying.value = false;
  });
  videoRef.value.addEventListener('timeupdate', () => {
    currentTime.value = videoRef.value?.currentTime || 0;
  });
  videoRef.value.addEventListener('loadedmetadata', () => {
    log('video:loadedmetadata', { duration: videoRef.value?.duration });
    duration.value = videoRef.value?.duration || 0;
  });
  videoRef.value.addEventListener('click', (e) => {
    log('video:click', { x: e.clientX, y: e.clientY });
    togglePlay(e);
  });
  document.addEventListener('fullscreenchange', () => {
    isFullscreen.value = !!document.fullscreenElement;
  });
});

// Cleanup
onUnmounted(() => {
  if (controlsTimeout.value) {
    clearTimeout(controlsTimeout.value);
  }
});
</script>

<template>
  <div
    class="relative bg-slate-900 rounded-lg overflow-hidden group min-w-[300px]"
    @mousemove="showControlsTemporarily"
    @mouseleave="showControls = false"
  >
    <!-- Video element -->
    <video
      ref="videoRef"
      :src="props.src"
      class="relative w-full h-full object-contain rounded-lg"
      preload="metadata"
      controlsList="nodownload"
    ></video>

    <!-- Video Controls -->
    <div
      class="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent p-4 transition-opacity duration-300 flex flex-col gap-1"
      :class="[showControls || !isPlaying ? 'opacity-100' : 'opacity-0']"
    >
      <!-- Progress Bar Container -->
      <div class="py-2">
        <ProgressBar
          :current-time="currentTime"
          :duration="duration"
          @seek="handleSeek"
          @dragstart="handleDragStart"
          @dragend="handleDragEnd"
        />
      </div>

      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <!-- Play/Pause Button -->
          <button
            @click="togglePlay"
            class="text-white hover:text-blue-500 transition-colors"
            type="button"
          >
            <svg v-if="!isPlaying" class="w-6 h-6" viewBox="0 0 24 24" fill="currentColor">
              <path d="M8 5v14l11-7z" />
            </svg>
            <svg v-else class="w-6 h-6" viewBox="0 0 24 24" fill="currentColor">
              <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
            </svg>
          </button>

          <!-- Volume Control -->
          <div class="flex items-center gap-2">
            <svg class="w-5 h-5 text-white" viewBox="0 0 24 24" fill="currentColor">
              <path
                d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"
              />
            </svg>
            <input
              type="range"
              min="0"
              max="1"
              step="0.1"
              :value="volume"
              @input="updateVolume"
              class="w-20 h-1 accent-blue-500"
            />
          </div>

          <!-- Time Display -->
          <div class="text-white text-sm">
            {{ formatTime(currentTime) }} / {{ formatTime(duration) }}
          </div>
        </div>

        <!-- Fullscreen Button -->
        <button
          @click="toggleFullscreen"
          class="text-white hover:text-blue-500 transition-colors"
          type="button"
        >
          <svg v-if="!isFullscreen" class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
            <path d="M7 14H5v5h5v-2H7v-3zm-2-4h2V7h3V5H5v5zm12 7h-3v2h5v-5h-2v3zM14 5v2h3v3h2V5h-5z" />
          </svg>
          <svg v-else class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
            <path d="M5 16h3v3h2v-5H5v2zm3-8H5v2h5V5H8v3zm6 11h2v-3h3v-2h-5v5zm2-11V5h-2v5h5V8h-3z" />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>