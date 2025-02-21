<!-- LoginView.vue -->
<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import logo from '@/assets/logo.svg';

const router = useRouter();
const email = ref('');
const password = ref('');

const handleLogin = () => {
  // Add your login logic here
  console.log('Login attempt:', { email: email.value, password: password.value });
  // On successful login:
  // router.push('/');
};
</script>

<template>
  <div class="min-h-screen w-full flex items-center justify-center bg-slate-900">
    <div class="flex flex-col gap-4 w-full max-w-md p-8">
      <!-- Logo/Brand -->
      <div class="flex flex-col gap-2 items-center">
        <img :src="logo" alt="NosDesk Logo" class="px-8" />
        <p class="text-slate-400 mt-2">Sign in to your account</p>
      </div>

      <!-- Login Form -->
      <form @submit.prevent="handleLogin" class="flex flex-col gap-4 space-y-6">
        <div>
          <label for="email" class="block text-sm font-medium text-slate-300">Email</label>
          <input
            id="email"
            v-model="email"
            type="email"
            required
            class="mt-1 block w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
            placeholder="Enter your email"
          />
        </div>

        <div>
          <label for="password" class="block text-sm font-medium text-slate-300">Password</label>
          <input
            id="password"
            v-model="password"
            type="password"
            required
            class="mt-1 block w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
            placeholder="Enter your password"
          />
        </div>

        <div class="flex items-center justify-between">
          <div class="flex items-center">
            <input
              id="remember-me"
              type="checkbox"
              class="h-4 w-4 rounded border-slate-700 bg-slate-800 text-blue-500 focus:ring-blue-500 focus:ring-offset-slate-900"
            />
            <label for="remember-me" class="ml-2 block text-sm text-slate-300">Remember me</label>
          </div>

          <a href="#" class="text-sm text-blue-500 hover:text-blue-400">Forgot password?</a>
        </div>

        <button
          type="submit"
          class="w-full flex justify-center py-2 px-4 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 focus:ring-offset-slate-900"
        >
          Sign in
        </button>
      </form>
    </div>
  </div>
</template>
