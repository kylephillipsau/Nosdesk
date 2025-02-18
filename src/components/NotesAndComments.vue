<script setup lang="ts">
import { ref, computed } from "vue";
import UserAvatar from "./UserAvatar.vue";

// Define NoteOrComment interface with attachments
interface NoteOrComment {
    id: number;
    content: string;
    author: string;
    createdAt: string;
    attachments?: { url: string; name: string }[];
}

// Mock data for notes and comments
const initialNotesAndComments: NoteOrComment[] = [
    { id: 1, content: "This needs follow-up ASAP", author: "Scott Mann", createdAt: "2023-10-01T14:48:00" },
    { id: 2, content: "Customer called regarding this issue", author: "Sam Martin", createdAt: "2023-10-02T09:15:00" }
];

// Reactive data for notes and comments
const notesAndComments = ref<NoteOrComment[]>(initialNotesAndComments);
const newNoteContent = ref<string>("");
const newAttachments = ref<{ url: string; name: string }[]>([]);

// Function to add a new note or comment with attachments
const addNoteOrComment = () => {
    if (newNoteContent.value.trim() || newAttachments.value.length > 0) {
        notesAndComments.value.push({
            id: Date.now(),
            content: newNoteContent.value,
            author: "Current User",
            createdAt: new Date().toISOString(),
            attachments: [...newAttachments.value] // Add attachments
        });
        newNoteContent.value = "";
        newAttachments.value = []; // Clear attachments after adding
    }
};

// Handle file upload with local storage until backend is completed
const handleFileUpload = (event: Event) => {
    const input = event.target as HTMLInputElement;
    if (input.files) {
        const files = Array.from(input.files);
        newAttachments.value = files.map(file => ({
            url: URL.createObjectURL(file), // Creates a blob URL for the file
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
    <div class="flex flex-col bg-gray-800 rounded-2xl p-6 gap-4 shadow-lg">
        <h2 class="text-lg font-medium text-gray-100">Notes and Comments</h2>

        <!-- List of Notes and Comments -->
        <div class="flex flex-col gap-2 space-y-3">
            <div v-for="item in notesAndComments" :key="item.id"
                class="flex flex-col gap-1 bg-gray-700 p-3 rounded-xl shadow-inner">
                <div class="flex flex-row gap-2">
                    <UserAvatar :name="'Stinky Man'" :showName="false" />
                    <div class="flex-grow">
                        <p class="text-gray-200">{{ item.content }}</p>
                        <small class="text-gray-500">{{ item.author }} - {{ formattedDate(item.createdAt) }}</small>
                    </div>
                </div>
                <div v-if="item.attachments && item.attachments.length > 0" class="mt-2">
                    <img v-for="attachment in item.attachments" :key="attachment.url" :src="attachment.url"
                        :alt="attachment.name" class="w-full rounded-lg object-cover mb-2">
                </div>
            </div>
        </div>

        <!-- Add New Note/Comment Form -->
        <div class="bg-gray-700 p-3 rounded-xl shadow-inner">
            <form @submit.prevent="addNoteOrComment" class="flex flex-col gap-4">
                <div class="relative">
                    <textarea v-model="newNoteContent"
                        class="w-full bg-gray-900 text-gray-100 border border-gray-600 rounded-md p-3 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
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