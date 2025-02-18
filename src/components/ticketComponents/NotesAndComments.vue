<!-- NotesAndComments.vue -->
<script setup lang="ts">
import { ref } from "vue";
import UserAvatar from "@/components/UserAvatar.vue";

interface NoteOrComment {
  id: number;
  content: string;
  author: string;
  createdAt: string;
  attachments?: { url: string; name: string }[];
}

const props = defineProps<{
  notesAndComments: NoteOrComment[];
}>();

// Reactive data for new note and attachments
const newNoteContent = ref<string>("");
const newAttachments = ref<{ url: string; name: string }[]>([]);

// Emits for adding a new note or comment with attachments
const emit = defineEmits<{
  (e: 'addNoteOrComment', value: { content: string, attachments: { url: string; name: string }[] }): void
}>();

// Function to add a new note or comment with attachments
const addNoteOrComment = () => {
  if (newNoteContent.value.trim() || newAttachments.value.length > 0) {
    emit('addNoteOrComment', {
      content: newNoteContent.value,
      attachments: [...newAttachments.value]
    });
    newNoteContent.value = "";
    newAttachments.value = [];
  }
};

// Handle file upload with local storage until backend is completed
const handleFileUpload = (event: Event) => {
  const input = event.target as HTMLInputElement;
  if (input.files) {
    const files = Array.from(input.files);
    newAttachments.value = files.map(file => ({
      url: URL.createObjectURL(file),
      name: file.name
    }));
  }
};

// Custom date formatting function
const formattedDate = (dateString: string): string => {
  const date = new Date(dateString);
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
};
</script>

<template>
  <div class="flex flex-col bg-slate-800 rounded-2xl p-6 gap-4 shadow-lg h-fit">
    <h2 class="text-lg font-medium text-slate-100">Notes and Comments</h2>

    <!-- List of Notes and Comments -->
    <div class="flex flex-col gap-2 space-y-3">
      <div v-for="item in props.notesAndComments" :key="item.id"
           class="flex flex-col gap-1 bg-slate-700 p-3 rounded-xl shadow-inner">
        <div class="flex flex-row gap-2">
          <UserAvatar :name="item.author" :showName="false" />
          <div class="flex-grow">
            <p class="text-slate-200">{{ item.content }}</p>
            <small class="text-slate-500">{{ item.author }} - {{ formattedDate(item.createdAt) }}</small>
          </div>
        </div>
        <div v-if="item.attachments && item.attachments.length > 0" class="mt-2">
          <img v-for="attachment in item.attachments" :key="attachment.url" :src="attachment.url"
               :alt="attachment.name" class="w-50 rounded-lg object-cover mb-2">
        </div>
      </div>
    </div>

    <!-- Add New Note/Comment Form -->
    <div class="bg-slate-700 p-3 rounded-xl shadow-inner">
      <form @submit.prevent="addNoteOrComment" class="flex flex-col gap-4">
        <div class="relative">
          <textarea v-model="newNoteContent"
                    class="w-full bg-slate-800 text-slate-100 border border-slate-600 rounded-md p-3 placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
                    placeholder="Add a new note or comment..." rows="3"></textarea>
          <label class="absolute bottom-3 right-3 cursor-pointer">
            <input type="file" @change="handleFileUpload" multiple class="hidden" />
            <span class="text-blue-500 hover:text-blue-600">Attach files</span>
          </label>
        </div>
        <div v-if="newAttachments.length > 0" class="mt-2">
          <img v-for="attachment in newAttachments" :key="attachment.url" :src="attachment.url"
               :alt="attachment.name" class="w-full rounded-lg object-cover mb-2">
        </div>
        <button type="submit" class="bg-blue-500 text-white p-2 rounded-md hover:bg-blue-600 transition-colors">
          Add
        </button>
      </form>
    </div>
  </div>
</template>