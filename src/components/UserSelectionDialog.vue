<script setup lang="ts">
import { ref, computed } from 'vue';

const props = defineProps<{
    isOpen: boolean;
    users: {
        id: string; 
        name: string; 
        email: string; 
        role: string;
    }[];
    selectedUserId?: string;
}>();

const emit = defineEmits<{
    (e: 'update:isOpen', value: boolean): void;
    (e: 'selectUser', value: string): void;
}>();

const closeDialog = () => {
    emit('update:isOpen', false);
};

const selectUser = (userId: string) => {
    emit('selectUser', userId);
    closeDialog();
};

const searchQuery = ref('');

// Computed property to filter users based on search query
const filteredUsers = computed(() => {
    const query = searchQuery.value.toLowerCase();
    return props.users.filter(user => 
        user.name.toLowerCase().includes(query) || 
        user.email.toLowerCase().includes(query) || 
        user.role.toLowerCase().includes(query)
    );
});
</script>

<template>
    <div v-if="isOpen" class="fixed inset-0 flex items-center justify-center z-50">
        <!-- Overlay -->
        <div class="absolute inset-0 bg-slate-900 opacity-50" @click="closeDialog"></div>

        <!-- Dialog Content -->
        <div class="bg-slate-800 rounded-lg shadow-xl p-4 max-w-md w-full mx-auto flex flex-col z-60">
            <div class="flex justify-between items-center mb-2">
                <h2 class="text-lg font-medium text-slate-100">Select User</h2>
                <input type="text" v-model="searchQuery" placeholder="Search..." class="ml-2 px-2 py-1 text-sm rounded bg-slate-700 text-slate-200">
            </div>

            <!-- User List -->
            <ul class="flex flex-col gap-1 overflow-y-auto max-h-60">
                <li v-for="user in filteredUsers" :key="user.id" 
                    @click="selectUser(user.id)"
                    class="p-2 rounded-md cursor-pointer hover:bg-slate-700 transition-colors text-slate-200">
                    <div class="flex justify-between items-center">
                        <div class="flex flex-col">
                            <span class="font-medium">{{ user.name }}</span>
                            <span class="text-xs text-slate-400">{{ user.role }}</span>
                        </div>
                        <span class="text-xs text-slate-500">{{ user.email }}</span>
                    </div>
                </li>
            </ul>

            <!-- Close Button -->
            <button @click="closeDialog"
                class="mt-4 px-3 py-1 text-sm bg-blue-500 text-white rounded-md hover:bg-blue-600 transition-colors">
                Close
            </button>
        </div>
    </div>
</template>