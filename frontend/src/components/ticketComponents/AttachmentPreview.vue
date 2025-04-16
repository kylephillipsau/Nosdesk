<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount, nextTick } from 'vue';
import AudioPlayer from "@/components/ticketComponents/AudioPlayer.vue";
import VideoPlayer from "@/components/ticketComponents/VideoPlayer.vue";
import FilePreview from "@/components/ticketComponents/FilePreview.vue";
import Modal from "@/components/Modal.vue";
import UserAvatar from "@/components/UserAvatar.vue";

interface Props {
  attachment: { id?: number; url: string; name: string; comment_id?: number };
  author: string;
  timestamp: string;
  isNew?: boolean;
  showDelete?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  isNew: false,
  showDelete: false
});

const emit = defineEmits<{
  (e: 'delete'): void;
  (e: 'submit'): void;
  (e: 'preview', src: string): void;
}>();

const showPreviewModal = ref(false);
const previewImageSrc = ref('');
const convertedThumbnailSrc = ref<string | null>(null);
const isLoadingThumbnail = ref(false);
const showPlaceholder = ref(false);
const conversionProgress = ref(0);

const log = (event: string, details?: any) => {
  console.log(`[AttachmentPreview] ${event}`, details || '');
};

// Add a more detailed debug logging function
const debugLog = (event: string, details?: any) => {
  console.log(`[AttachmentPreview:DEBUG] ${event}`, details || '', new Date().toISOString());
};

// Add DOM visibility logging
const logVisibility = async (elementDesc: string) => {
  await nextTick();
  debugLog(`DOM UPDATE CHECK - ${elementDesc}`, {
    isLoadingThumbnail: isLoadingThumbnail.value,
    showPlaceholder: showPlaceholder.value,
    hasConvertedThumbnail: !!convertedThumbnailSrc.value,
    containerExists: document.querySelector('.heic-container') !== null,
    placeholderVisible: document.querySelector('.heic-placeholder') !== null,
    loadingVisible: document.querySelector('.heic-loading') !== null,
  });
};

const isVideoFile = (filename: string): boolean => {
  try {
    const videoExtensions = ['.mp4', '.mov', '.webm', '.avi', '.mkv'];
    return videoExtensions.some(ext => filename.toLowerCase().endsWith(ext));
  } catch (error) {
    console.error('Error checking if file is a video:', error);
    return false;
  }
};

const isAudioFile = (filename: string): boolean => {
  try {
    const audioExtensions = ['.mp3', '.wav', '.ogg', '.m4a', '.webm'];
    return audioExtensions.some(ext => filename.toLowerCase().endsWith(ext)) || 
           (typeof filename === 'string' && filename.toLowerCase().includes('voice note'));
  } catch (error) {
    console.error('Error checking if file is audio:', error);
    return false;
  }
};

const isImageFile = (filename: string): boolean => {
  try {
    const imageExtensions = ['.jpg', '.jpeg', '.png', '.gif', '.webp', '.svg', '.heic'];
    return imageExtensions.some(ext => filename.toLowerCase().endsWith(ext));
  } catch (error) {
    console.error('Error checking if file is an image:', error);
    return false;
  }
};

const isHeicFile = (filename: string): boolean => {
  try {
    if (!filename) return false;
    return filename.toLowerCase().endsWith('.heic');
  } catch (error) {
    console.error('Error checking if file is HEIC:', error);
    return false;
  }
};

const attachmentType = computed(() => {
  // Safety check for null/undefined attachment name
  if (!props.attachment?.name) {
    return 'file';
  }
  
  if (isAudioFile(props.attachment.name)) return 'audio';
  if (isVideoFile(props.attachment.name)) return 'video';
  if (isImageFile(props.attachment.name)) return 'image';
  return 'file';
});

const openImagePreview = async (src: string, retryCount = 0) => {
  // If we already have a converted image, just show it
  if (previewImageSrc.value && previewImageSrc.value !== props.attachment.url) {
    showPreviewModal.value = true;
    emit('preview', previewImageSrc.value);
    return;
  }
  
  // If it's a HEIC file, we need to convert it first
  if (isHeicFile(props.attachment.name)) {
    try {
      // If we already have a converted thumbnail, use that instead of converting again
      if (convertedThumbnailSrc.value) {
        previewImageSrc.value = convertedThumbnailSrc.value;
        showPreviewModal.value = true;
        emit('preview', previewImageSrc.value);
        return;
      }
      
      log('Converting HEIC image for preview');
      
      // Dynamically import heic2any only when needed
      const heic2any = await import('heic2any');
      
      // Fetch the HEIC file
      const response = await fetch(src);
      if (!response.ok) {
        throw new Error(`Failed to fetch HEIC file: ${response.status} ${response.statusText}`);
      }
      
      const blob = await response.blob();
      if (!blob || blob.size === 0) {
        throw new Error('Received empty blob from server');
      }
      
      // Convert HEIC to JPEG
      const jpegBlob = await heic2any.default({
        blob,
        toType: 'image/jpeg',
        quality: 0.8
      }) as Blob;
      
      // Create a URL for the converted image
      const convertedSrc = URL.createObjectURL(jpegBlob);
      previewImageSrc.value = convertedSrc;
      log('HEIC image converted successfully for preview');
    } catch (error) {
      console.error('Error converting HEIC image for preview:', error);
      
      // Retry up to 2 times with increasing delay
      if (retryCount < 2) {
        const retryDelay = 500 * (retryCount + 1);
        log(`Retrying HEIC preview conversion in ${retryDelay}ms (attempt ${retryCount + 1}/2)`);
        
        setTimeout(() => {
          openImagePreview(src, retryCount + 1).catch(err => {
            console.error(`Preview retry ${retryCount + 1} failed:`, err);
          });
        }, retryDelay);
        return;
      }
      
      // Fallback to original source if conversion fails
      previewImageSrc.value = src;
    }
  } else {
    previewImageSrc.value = src;
  }
  
  showPreviewModal.value = true;
  emit('preview', previewImageSrc.value);
};

const closeImagePreview = () => {
  showPreviewModal.value = false;
  
  // Don't clean up the object URL when closing the modal
  // We'll keep it for subsequent opens
  // Only clean up on component unmount
};

// Add a computed property to check if a HEIC image is currently converting
const isHeicConverting = computed(() => {
  return isHeicFile(props.attachment.name) && (!convertedThumbnailSrc.value || isLoadingThumbnail.value);
});

// Add a computed property to check if an image is ready to be displayed
const isImageReady = computed(() => {
  return !isHeicFile(props.attachment.name) || 
         (isHeicFile(props.attachment.name) && convertedThumbnailSrc.value && !isLoadingThumbnail.value);
});

// Load HEIC thumbnails on component mount
onMounted(() => {
  debugLog('Component onMounted hook started');
  
  if (isHeicFile(props.attachment.name)) {
    // Set loading state immediately on mount
    isLoadingThumbnail.value = true;
    showPlaceholder.value = true;
    conversionProgress.value = 5; // Start progress indicator
    debugLog('Set loading state and placeholder to true immediately');
    
    // Start conversion immediately
    convertHeicThumbnail().catch(err => {
      console.error('Failed to convert HEIC thumbnail:', err);
      isLoadingThumbnail.value = false;
    });
  }
});

// Clean up object URLs when component is unmounted
onBeforeUnmount(() => {
  debugLog('Component unmounting, cleaning up resources');
  
  if (convertedThumbnailSrc.value) {
    URL.revokeObjectURL(convertedThumbnailSrc.value);
    debugLog('Revoked thumbnail URL');
  }
  
  if (previewImageSrc.value && previewImageSrc.value !== props.attachment.url) {
    URL.revokeObjectURL(previewImageSrc.value);
    debugLog('Revoked preview image URL');
  }
});

// Add a method to convert HEIC to JPEG for thumbnails
const convertHeicThumbnail = async (retryCount = 0) => {
  if (!isHeicFile(props.attachment.name)) return;
  
  try {
    // Set initial state
    isLoadingThumbnail.value = true;
    showPlaceholder.value = true;
    conversionProgress.value = 10;
    
    // Import heic2any library
    const heic2any = await import('heic2any');
    conversionProgress.value = 20;
    
    // Fetch the HEIC file
    const response = await fetch(props.attachment.url);
    if (!response.ok) {
      throw new Error(`Failed to fetch HEIC file: ${response.status} ${response.statusText}`);
    }
    
    conversionProgress.value = 40;
    const blob = await response.blob();
    if (!blob || blob.size === 0) {
      throw new Error('Received empty blob from server');
    }
    
    conversionProgress.value = 50;
    
    // Convert HEIC to JPEG
    conversionProgress.value = 60;
    const jpegBlob = await heic2any.default({
      blob,
      toType: 'image/jpeg',
      quality: 0.5
    }) as Blob;
    
    conversionProgress.value = 90;
    
    // Create a URL for the converted image
    convertedThumbnailSrc.value = URL.createObjectURL(jpegBlob);
    conversionProgress.value = 100;
    log('HEIC thumbnail converted successfully');
    
    // Add a small delay before hiding the loading indicator
    setTimeout(() => {
      isLoadingThumbnail.value = false;
    }, 300);
  } catch (error) {
    console.error('Error converting HEIC thumbnail:', error);
    conversionProgress.value = 0;
    
    // Retry up to 2 times with increasing delay
    if (retryCount < 2) {
      const retryDelay = 500 * (retryCount + 1);
      log(`Retrying HEIC conversion in ${retryDelay}ms (attempt ${retryCount + 1}/2)`);
      
      setTimeout(() => {
        convertHeicThumbnail(retryCount + 1).catch(err => {
          console.error(`Retry ${retryCount + 1} failed:`, err);
        });
      }, retryDelay);
      return;
    }
    
    // All retries failed
    convertedThumbnailSrc.value = null;
    isLoadingThumbnail.value = false;
  } finally {
    // Keep the placeholder visible if conversion failed
    showPlaceholder.value = !convertedThumbnailSrc.value;
  }
};
</script>

<template>
  <div :class="[
    'flex flex-col gap-2',
    attachmentType === 'audio' ? 'w-full' : '',
    attachmentType === 'video' ? 'bg-slate-800 rounded-lg p-3 w-full' : '',
    attachmentType === 'image' ? 'max-w-[250px]' : ''
  ]">
    <!-- Audio/Video header -->
    <template v-if="attachmentType === 'audio' || attachmentType === 'video'">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <template v-if="attachmentType === 'audio'">
            <UserAvatar 
              :name="author" 
              :showName="false" 
              v-if="author"
            />
            <div v-else class="w-6 h-6 rounded-full bg-slate-600"></div>
            <div class="flex flex-col">
              <span class="text-sm text-slate-200">{{ attachment.name }}</span>
            </div>
          </template>
          <template v-if="attachmentType === 'video'">
            <svg class="w-5 h-5 text-slate-400" viewBox="0 0 20 20" fill="currentColor">
              <path d="M2 6a2 2 0 012-2h6a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V6zM14.553 7.106A1 1 0 0014 8v4a1 1 0 00.553.894l2 1A1 1 0 0018 13V7a1 1 0 00-1.447-.894l-2 1z" />
            </svg>
            <span class="text-sm text-slate-300">{{ attachment.name }}</span>
          </template>
        </div>
        <div class="flex items-center gap-2">
          <!-- Download button -->
          <a
            :href="attachment.url"
            target="_blank"
            :download="attachment.name"
            class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-700 rounded transition-colors"
            title="Download attachment"
            @click.stop
          >
            <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clip-rule="evenodd" />
            </svg>
          </a>
          <!-- Delete button -->
          <button
            v-if="showDelete"
            type="button"
            @click.stop="emit('delete')"
            class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-700 rounded transition-colors"
            :title="'Delete ' + attachmentType"
          >
            <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
            </svg>
          </button>
        </div>
      </div>
    </template>

    <!-- Content -->
    <template v-if="attachmentType === 'audio'">
      <div class="bg-slate-800 rounded-lg p-3 w-full">
        <AudioPlayer :src="attachment.url" />
      </div>
    </template>
    <template v-else-if="attachmentType === 'video'">
      <VideoPlayer
        :src="attachment.url"
        class="w-full h-64"
        :show-delete="showDelete"
        @delete="emit('delete')"
      />
    </template>
    <template v-else-if="attachmentType === 'image'">
      <!-- Always maintain a fixed size container for images, regardless of loading state -->
      <div class="relative group w-full min-w-42 h-58 rounded-lg overflow-hidden"
        :class="[
          isHeicConverting 
            ? 'bg-blue-900/90 border-2 border-blue-500' 
            : 'bg-slate-800'
        ]">
        <button
          v-if="showDelete"
          type="button"
          @click.stop="emit('delete')"
          class="absolute top-2 right-2 z-30 p-1.5 bg-slate-800/80 text-slate-400 hover:text-white hover:bg-slate-700 rounded transition-colors"
          title="Delete image"
        >
          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
          </svg>
        </button>
        
        <!-- Simplified HEIC conversion indicator - visible during conversion -->
        <div 
          v-if="isHeicConverting"
          class="absolute inset-0 flex flex-col items-center justify-center bg-slate-900/90 z-20"
        >
          <!-- Blue overlay with "CONVERTING" text -->
          <div class="absolute p-1 inset-0 flex items-center justify-center">
            <div class="bg-blue-600 px-2 py-2 text-center rounded-md shadow-lg">
              <span class="text-sm text-white font-black tracking-wider">Converting HEIC to JPEG...</span>
            </div>
          </div>
          
          <!-- Progress indicator at the bottom -->
          <div class="absolute bottom-0 left-0 right-0 bg-slate-800/90 p-2">
            <div class="w-full h-2 bg-slate-700 rounded-full overflow-hidden">
              <div 
                class="h-full bg-blue-500 shimmer" 
                :style="{ width: conversionProgress > 0 ? `${conversionProgress}%` : '100%' }"
              ></div>
            </div>
            <div class="text-xs text-blue-300 text-center mt-1">
              {{ conversionProgress > 0 ? `${conversionProgress}%` : 'Processing...' }}
            </div>
          </div>
        </div>
        
        <!-- Image with transition for HEIC -->
        <transition
          name="fade"
          mode="out-in"
          appear
        >
          <img 
            v-if="isImageReady"
            key="image"
            :src="isHeicFile(attachment.name) && convertedThumbnailSrc ? convertedThumbnailSrc : attachment.url" 
            :alt="attachment.name" 
            class="w-full h-full object-contain bg-slate-900/50 z-5 heic-image"
          >
        </transition>

        <!-- Preview hover overlay - only show when image is ready -->
        <div 
          v-if="isImageReady"
          class="absolute inset-0 bg-slate-900/30 opacity-0 group-hover:opacity-100 transition-opacity duration-200 flex items-center justify-center cursor-pointer z-25"
          @click.stop="() => {
            try {
              openImagePreview(attachment.url, 0);
            } catch (error) {
              console.error('Error opening image preview:', error);
              // Fallback to just showing the modal with the original image
              previewImageSrc = attachment.url;
              showPreviewModal = true;
            }
          }"
        >
          <svg class="w-8 h-8 text-white" viewBox="0 0 20 20" fill="currentColor">
            <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
            <path fill-rule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clip-rule="evenodd" />
          </svg>
        </div>
        <div 
          class="absolute top-0 left-0 p-2 bg-gradient-to-b from-slate-900/80 to-transparent w-full opacity-0 group-hover:opacity-100 transition-opacity duration-200 z-25"
        >
          <span class="text-sm text-white font-medium truncate block">{{ attachment.name }}</span>
        </div>
        <!-- Download button for images -->
        <div 
          class="absolute bottom-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200 z-25"
        >
          <div class="flex gap-2">
            <!-- Original file download button -->
            <a
              :href="attachment.url"
              target="_blank"
              :download="attachment.name"
              class="flex items-center gap-1 p-2 bg-slate-800/80 text-slate-400 hover:text-white hover:bg-slate-700 rounded transition-colors"
              :title="isHeicFile(attachment.name) ? 'Download original HEIC' : 'Download image'"
              @click.stop
            >
              <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clip-rule="evenodd" />
              </svg>
              <span v-if="isHeicFile(attachment.name)" class="text-xs font-medium">HEIC</span>
            </a>

            <!-- JPEG download button for HEIC files -->
            <a
              v-if="isHeicFile(attachment.name) && convertedThumbnailSrc"
              :href="convertedThumbnailSrc"
              target="_blank"
              :download="attachment.name.replace(/\.heic$/i, '.jpg')"
              class="flex items-center gap-1 p-2 bg-blue-600 text-white hover:bg-blue-700 rounded transition-colors"
              title="Download JPEG"
              @click.stop
            >
              <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clip-rule="evenodd" />
              </svg>
              <span class="text-xs font-medium">JPEG</span>
            </a>
          </div>
        </div>
      </div>
    </template>
    <template v-else>
      <FilePreview
        :src="attachment.url"
        :filename="attachment.name"
        :author="author"
        :timestamp="timestamp"
        :show-delete="showDelete"
        @delete="emit('delete')"
        @preview="(src) => emit('preview', src)"
      />
    </template>

    <!-- Submit controls for new attachments -->
    <div v-if="isNew && attachmentType === 'video'" class="flex justify-end gap-2 mt-2">
      <button
        type="button"
        @click.stop="emit('delete')"
        class="px-3 py-1.5 text-slate-300 hover:text-white transition-colors"
      >
        Cancel
      </button>
      <button
        type="button"
        @click.stop="emit('submit')"
        class="px-3 py-1.5 bg-green-500 text-white text-sm rounded hover:bg-green-600 transition-colors"
      >
        Submit Video
      </button>
    </div>
    
    <!-- Image Preview Modal -->
    <Modal 
      :show="showPreviewModal" 
      title="Image Preview" 
      @close="closeImagePreview"
    >
      <div class="flex flex-col items-center gap-1">
        <!-- Image preview with transition -->
        <transition name="fade" mode="out-in" appear>
          <img 
            :src="previewImageSrc" 
            :alt="attachment.name" 
            class="max-w-full max-h-[70vh] object-contain"
          />
        </transition>
        
        <div class="mt-4 text-center text-sm text-slate-300">
          {{ attachment.name }}
        </div>
        
        <div class="mt-4 flex gap-3">
          <!-- For HEIC files, show both download options -->
          <template v-if="isHeicFile(attachment.name) && previewImageSrc !== attachment.url">
            <!-- Original HEIC download button -->
            <a
              :href="attachment.url"
              target="_blank"
              :download="attachment.name"
              class="px-4 py-2 bg-slate-800 text-white text-sm rounded hover:bg-slate-700 transition-colors flex items-center gap-2"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path>
              </svg>
              Download original (HEIC)
            </a>
            
            <!-- Converted JPEG download button -->
            <a
              :href="previewImageSrc"
              target="_blank"
              :download="attachment.name.replace(/\.heic$/i, '.jpg')"
              class="px-4 py-2 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 transition-colors flex items-center gap-2"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path>
              </svg>
              Download converted (JPEG)
            </a>
          </template>
          
          <!-- For non-HEIC files, show the standard download button -->
          <a
            v-else
            :href="previewImageSrc"
            target="_blank"
            :download="attachment.name"
            class="px-4 py-2 bg-slate-800 text-white text-sm rounded hover:bg-slate-700 transition-colors flex items-center gap-2"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path>
            </svg>
            Download image
          </a>
        </div>
      </div>
    </Modal>
  </div>
</template>

<style scoped>
/* Add transition styles */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: scale(0.95);
}

.fade-enter-to,
.fade-leave-from {
  opacity: 1;
  transform: scale(1);
}

/* Add a pulse animation for the loading spinner */
@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}

.animate-pulse {
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

/* Add a shimmer effect for loading placeholders */
@keyframes shimmer {
  0% {
    background-position: -200% 0;
  }
  100% {
    background-position: 200% 0;
  }
}

.shimmer {
  background: linear-gradient(90deg, rgba(59, 130, 246, 0.6) 25%, rgba(96, 165, 250, 0.9) 50%, rgba(59, 130, 246, 0.6) 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

/* Add a pulsating effect for the loading container */
@keyframes pulse-bg {
  0%, 100% {
    background-color: rgba(15, 23, 42, 0.9); /* slate-900/90 */
    box-shadow: 0 0 15px rgba(59, 130, 246, 0.3);
  }
  50% {
    background-color: rgba(15, 23, 42, 0.8); /* slate-900/80 */
    box-shadow: 0 0 25px rgba(59, 130, 246, 0.5);
  }
}

/* Add a pulsing border animation */
@keyframes pulse-border {
  0%, 100% {
    border-color: rgba(59, 130, 246, 0.8);
  }
  50% {
    border-color: rgba(96, 165, 250, 0.4);
  }
}

/* Add a ping animation for the icon */
@keyframes ping {
  75%, 100% {
    transform: scale(1.2);
    opacity: 0;
  }
}

.animate-ping {
  animation: ping 1.5s cubic-bezier(0, 0, 0.2, 1) infinite;
}

/* Add a subtle floating animation */
@keyframes float {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-5px);
  }
}

/* Ensure z-index layering works correctly */
.heic-image {
  z-index: 5 !important;
}
</style>