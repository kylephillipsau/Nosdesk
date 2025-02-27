<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRouter } from "vue-router";
import UserAvatar from "@/components/UserAvatar.vue";
import StatusBadge from "@/components/StatusBadge.vue";

interface User {
  id: string;
  username: string;
  name: string;
  email: string;
  role: string;
  department: string;
  status: "active" | "inactive";
}

const router = useRouter();
const users = ref<User[]>([
  {
    id: "1",
    username: "john.doe",
    name: "John Doe",
    email: "john.doe@example.com",
    role: "Support Agent",
    department: "IT Support",
    status: "active",
  },
  {
    id: "2",
    username: "jane.smith",
    name: "Jane Smith",
    email: "jane.smith@example.com",
    role: "Support Manager",
    department: "IT Support",
    status: "active",
  },
]);

const searchQuery = ref("");

const filteredUsers = computed(() => {
  const query = searchQuery.value.toLowerCase();
  if (!query) return users.value;

  return users.value.filter(
    (user) =>
      user.name.toLowerCase().includes(query) ||
      user.email.toLowerCase().includes(query) ||
      user.role.toLowerCase().includes(query) ||
      user.department.toLowerCase().includes(query)
  );
});

const navigateToUser = (username: string) => {
  router.push(`/users/${username}`);
};

const fetchUsers = async () => {
  // TODO: Replace with actual API call
  const mockUsers: User[] = [
    {
      id: "1",
      username: "john.doe",
      name: "John Doe",
      email: "john.doe@example.com",
      role: "Support Agent",
      department: "IT Support",
      status: "active",
    },
    {
      id: "2",
      username: "jane.smith",
      name: "Jane Smith",
      email: "jane.smith@example.com",
      role: "Support Manager",
      department: "IT Support",
      status: "active",
    },
  ];
  users.value = mockUsers;
};

onMounted(() => {
  fetchUsers();
});
</script>

<template>
  <div class="min-h-screen bg-gray-900 text-white p-6 flex justify-center">
    <div class="flex flex-1 flex-col gap-4 max-w-7xl">
      <div class="flex items-center justify-between mb-6">
        <h1 class="text-2xl font-medium text-white">Users</h1>
        <div class="flex items-center gap-4">
          <div class="relative">
            <input
              v-model="searchQuery"
              type="text"
              placeholder="Search users..."
              class="w-64 bg-slate-800 text-white rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <div
              class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none"
            >
              <svg
                class="w-5 h-5 text-slate-400"
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 20 20"
                fill="currentColor"
              >
                <path
                  fill-rule="evenodd"
                  d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z"
                  clip-rule="evenodd"
                />
              </svg>
            </div>
          </div>
          <button
            class="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded-lg transition-colors"
          >
            Add User
          </button>
        </div>
      </div>

      <div class="bg-gray-800 rounded-lg overflow-hidden">
        <div class="overflow-x-auto">
          <table class="min-w-full divide-y divide-gray-700">
            <thead>
              <tr>
                <th
                  class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider"
                >
                  User
                </th>
                <th
                  class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider"
                >
                  Role
                </th>
                <th
                  class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider"
                >
                  Department
                </th>
                <th
                  class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider"
                >
                  Status
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-700">
              <tr
                v-for="user in filteredUsers"
                :key="user.id"
                @click="navigateToUser(user.username)"
                class="hover:bg-gray-700 cursor-pointer"
              >
                <td class="px-6 py-2 whitespace-nowrap">
                  <div class="flex flex-row gap-2 items-center">
                    <UserAvatar
                      :name="user.name"
                      size="sm"
                      :clickable="false"
                      :show-name="false"
                    />

                    <div class="ml-4">
                      <div class="text-sm font-medium">{{ user.name }}</div>
                      <div class="text-sm text-gray-400">{{ user.email }}</div>
                    </div>
                  </div>
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm">
                  {{ user.role }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm">
                  {{ user.department }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <StatusBadge
                    type="status"
                    :value="user.status === 'active' ? 'open' : 'closed'"
                  />
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>
