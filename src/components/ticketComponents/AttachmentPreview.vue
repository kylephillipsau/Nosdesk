<script setup lang="ts">
import { computed, ref } from 'vue';
import UserAvatar from "@/components/UserAvatar.vue";
import AudioPlayer from "@/components/ticketComponents/AudioPlayer.vue";
import VideoPlayer from "@/components/ticketComponents/VideoPlayer.vue";
import FilePreview from "@/components/ticketComponents/FilePreview.vue";
import Modal from "@/components/Modal.vue";

interface Props {
  attachment: { url: string; name: string };
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

const log = (event: string, details?: any) => {
  console.log(`[AttachmentPreview] ${event}`, details || '');
};

const isVideoFile = (filename: string): boolean => {
  const videoExtensions = ['.mp4', '.mov', '.webm', '.avi', '.mkv'];
  return videoExtensions.some(ext => filename.toLowerCase().endsWith(ext));
};

const isAudioFile = (filename: string): boolean => {
  const audioExtensions = ['.mp3', '.wav', '.ogg', '.m4a', '.webm'];
  return audioExtensions.some(ext => filename.toLowerCase().endsWith(ext)) || filename.toLowerCase().includes('voice note');
};

const isImageFile = (filename: string): boolean => {
  const imageExtensions = ['.jpg', '.jpeg', '.png', '.gif', '.webp', '.svg'];
  return imageExtensions.some(ext => filename.toLowerCase().endsWith(ext));
};

const attachmentType = computed(() => {
  if (isAudioFile(props.attachment.name)) return 'audio';
  if (isVideoFile(props.attachment.name)) return 'video';
  if (isImageFile(props.attachment.name)) return 'image';
  return 'file';
});

const openImagePreview = (src: string) => {
  previewImageSrc.value = src;
  showPreviewModal.value = true;
  emit('preview', src);
};

const closeImagePreview = () => {
  showPreviewModal.value = false;
};
</script>

<template>
  <div :class="[
    'flex flex-col gap-2',
    attachmentType === 'video' ? 'bg-slate-800 rounded-lg p-3' : ''
  ]">
    <!-- Audio/Video header -->
    <template v-if="attachmentType === 'audio' || attachmentType === 'video'">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <template v-if="attachmentType === 'audio'">
            <UserAvatar :name="author" :showName="false" />
            <div class="flex flex-col">
              <span class="text-sm text-slate-200">{{ author }}</span>
              <span class="text-xs text-slate-400">{{ timestamp }}</span>
            </div>
          </template>
          <template v-if="attachmentType === 'video'">
            <svg class="w-5 h-5 text-slate-400" viewBox="0 0 20 20" fill="currentColor">
              <path d="M2 6a2 2 0 012-2h6a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V6zM14.553 7.106A1 1 0 0014 8v4a1 1 0 00.553.894l2 1A1 1 0 0018 13V7a1 1 0 00-1.447-.894l-2 1z" />
            </svg>
            <span class="text-sm text-slate-300">{{ attachment.name }}</span>
          </template>
        </div>
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
    </template>

    <!-- Content -->
    <template v-if="attachmentType === 'audio'">
      <AudioPlayer :src="attachment.url" class="w-full" />
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
      <div class="relative group w-full h-64 rounded-lg overflow-hidden">
        <button
          v-if="showDelete"
          type="button"
          @click.stop="emit('delete')"
          class="absolute top-2 right-2 z-10 p-1.5 bg-slate-800/80 text-slate-400 hover:text-white hover:bg-slate-700 rounded transition-colors"
          title="Delete image"
        >
          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
          </svg>
        </button>
        <img 
          :src="attachment.url" 
          :alt="attachment.name" 
          class="w-full h-full object-contain bg-slate-900/50"
        >
        <div 
          class="absolute inset-0 bg-slate-900/30 opacity-0 group-hover:opacity-100 transition-opacity duration-200 flex items-center justify-center cursor-pointer"
          @click.stop="openImagePreview(attachment.url)"
        >
          <svg class="w-8 h-8 text-white" viewBox="0 0 20 20" fill="currentColor">
            <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
            <path fill-rule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clip-rule="evenodd" />
          </svg>
        </div>
        <div 
          class="absolute top-0 left-0 p-2 bg-gradient-to-b from-slate-900/80 to-transparent w-full opacity-0 group-hover:opacity-100 transition-opacity duration-200"
        >
          <span class="text-sm text-white font-medium truncate block">{{ attachment.name }}</span>
        </div>
      </div>
    </template>
    <template v-else>
      <FilePreview
        :src="attachment.url"
        :filename="attachment.name"
        :author="author"
        :timestamp="timestamp"
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
      <div class="flex flex-col items-center">
        <img 
          :src="previewImageSrc" 
          :alt="attachment.name" 
          class="max-w-full max-h-[70vh] object-contain"
        />
        <div class="mt-4 text-center text-sm text-slate-300">
          {{ attachment.name }}
        </div>
      </div>
    </Modal>
  </div>
</template>