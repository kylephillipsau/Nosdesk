<!-- VideoPlayer.vue -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

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
const isDragging = ref(false);

const formatTime = (seconds: number): string => {
  if (!seconds || isNaN(seconds)) return "0:00";
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
};

const togglePlay = (event?: Event) => {
  if (event) {
    event.preventDefault();
    event.stopPropagation();
  }
  if (!videoRef.value) return;
  if (isPlaying.value) {
    videoRef.value.pause();
  } else {
    videoRef.value.play();
  }
};

const handleMouseDown = (event: MouseEvent) => {
  if (!videoRef.value || !duration.value) return;
  isDragging.value = true;
  updateProgress(event);
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
};

const handleMouseMove = (event: MouseEvent) => {
  if (!isDragging.value) return;
  updateProgress(event);
};

const handleMouseUp = () => {
  isDragging.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', handleMouseUp);
};

const updateProgress = (event: MouseEvent) => {
  if (!videoRef.value || !duration.value) return;
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const x = Math.max(0, Math.min(event.clientX - rect.left, rect.width));
  const percentage = x / rect.width;
  videoRef.value.currentTime = percentage * duration.value;
};

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

const updateVolume = (event: Event) => {
  const input = event.target as HTMLInputElement;
  volume.value = Number(input.value);
  if (videoRef.value) {
    videoRef.value.volume = volume.value;
  }
};

const showControlsTemporarily = () => {
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

onMounted(() => {
  if (!videoRef.value) return;

  videoRef.value.addEventListener('play', () => isPlaying.value = true);
  videoRef.value.addEventListener('pause', () => isPlaying.value = false);
  videoRef.value.addEventListener('timeupdate', () => {
    currentTime.value = videoRef.value?.currentTime || 0;
  });
  videoRef.value.addEventListener('loadedmetadata', () => {
    duration.value = videoRef.value?.duration || 0;
  });
  
  document.addEventListener('fullscreenchange', () => {
    isFullscreen.value = !!document.fullscreenElement;
  });
});

onUnmounted(() => {
  if (controlsTimeout.value) {
    clearTimeout(controlsTimeout.value);
  }
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', handleMouseUp);
});
</script>

<template>
  <div 
    class="relative bg-slate-900 rounded-lg overflow-hidden group"
    @mousemove.stop="showControlsTemporarily"
    @mouseleave.stop="showControls = false"
  >
    <video
      ref="videoRef"
      :src="props.src"
      class="w-full rounded-lg"
      @click.stop="togglePlay"
      preload="metadata"
      controlsList="nodownload"
    ></video>
    
    <!-- Video Controls -->
    <div 
      class="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent p-4 transition-opacity duration-300"
      :class="[showControls || !isPlaying ? 'opacity-100' : 'opacity-0']"
      @click.stop
    >
      <!-- Progress Bar -->
      <div 
        class="w-full h-1 bg-slate-600 rounded-full mb-4 cursor-pointer relative"
        @mousedown.stop="handleMouseDown"
      >
        <div 
          class="h-full bg-blue-500 rounded-full relative"
          :style="{ width: `${(currentTime / duration) * 100}%` }"
        >
          <div 
            class="absolute right-0 top-1/2 -translate-y-1/2 w-3 h-3 bg-blue-500 rounded-full shadow-lg"
            :class="{ 'scale-150': isDragging }"
          ></div>
        </div>
      </div>
      
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <!-- Play/Pause Button -->
          <button
            @click.stop="togglePlay"
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
          <div class="flex items-center gap-2" @click.stop>
            <svg class="w-5 h-5 text-white" viewBox="0 0 24 24" fill="currentColor">
              <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/>
            </svg>
            <input
              type="range"
              min="0"
              max="1"
              step="0.1"
              :value="volume"
              @input.stop="updateVolume"
              class="w-20 accent-blue-500"
            >
          </div>
          
          <!-- Time Display -->
          <div class="text-white text-sm">
            {{ formatTime(currentTime) }} / {{ formatTime(duration) }}
          </div>
        </div>
        
        <!-- Fullscreen Button -->
        <button
          @click.stop="toggleFullscreen"
          class="text-white hover:text-blue-500 transition-colors"
          type="button"
        >
          <svg v-if="!isFullscreen" class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
            <path d="M7 14H5v5h5v-2H7v-3zm-2-4h2V7h3V5H5v5zm12 7h-3v2h5v-5h-2v3zM14 5v2h3v3h2V5h-5z"/>
          </svg>
          <svg v-else class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
            <path d="M5 16h3v3h2v-5H5v2zm3-8H5v2h5V5H8v3zm6 11h2v-3h3v-2h-5v5zm2-11V5h-2v5h5V8h-3z"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template> 