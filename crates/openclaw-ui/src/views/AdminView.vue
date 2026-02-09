<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useApi } from '@/composables/useApi'
import type { User } from '@/stores/auth'

const api = useApi()

const activeTab = ref<'users' | 'system'>('users')

// Users state
const users = ref<User[]>([])
const loadingUsers = ref(true)
const showUserForm = ref(false)
const editingUser = ref<User | null>(null)
const userForm = ref({
  username: '',
  password: '',
  email: '',
  role: 'viewer' as 'admin' | 'operator' | 'viewer',
})
const userFormError = ref('')
const savingUser = ref(false)

// System state
const systemHealth = ref<Record<string, unknown> | null>(null)
const systemVersion = ref<{ version: string; name: string } | null>(null)
const loadingSystem = ref(true)

onMounted(async () => {
  await Promise.all([loadUsers(), loadSystem()])
})

async function loadUsers() {
  loadingUsers.value = true
  try {
    const result = await api.call<{ users: User[] }>('users.list')
    users.value = result.users
  } finally {
    loadingUsers.value = false
  }
}

async function loadSystem() {
  loadingSystem.value = true
  try {
    const [health, version] = await Promise.all([
      api.call<Record<string, unknown>>('system.health'),
      api.call<{ version: string; name: string }>('system.version'),
    ])
    systemHealth.value = health
    systemVersion.value = version
  } finally {
    loadingSystem.value = false
  }
}

function openCreateUser() {
  editingUser.value = null
  userForm.value = { username: '', password: '', email: '', role: 'viewer' }
  userFormError.value = ''
  showUserForm.value = true
}

function openEditUser(user: User) {
  editingUser.value = user
  userForm.value = {
    username: user.username,
    password: '',
    email: user.email || '',
    role: user.role,
  }
  userFormError.value = ''
  showUserForm.value = true
}

async function saveUser() {
  userFormError.value = ''
  savingUser.value = true

  try {
    if (editingUser.value) {
      // Update existing user
      await api.call('users.update', {
        id: editingUser.value.id,
        role: userForm.value.role,
        email: userForm.value.email || undefined,
      })
    } else {
      // Create new user
      if (!userForm.value.password) {
        userFormError.value = 'Password is required'
        return
      }
      await api.call('users.create', {
        username: userForm.value.username,
        password: userForm.value.password,
        role: userForm.value.role,
        email: userForm.value.email || undefined,
      })
    }
    showUserForm.value = false
    await loadUsers()
  } catch (e) {
    userFormError.value = e instanceof Error ? e.message : 'Failed to save user'
  } finally {
    savingUser.value = false
  }
}

async function deleteUser(user: User) {
  if (!confirm(`Are you sure you want to delete user "${user.username}"?`)) return

  try {
    await api.call('users.delete', { id: user.id })
    await loadUsers()
  } catch (e) {
    alert(e instanceof Error ? e.message : 'Failed to delete user')
  }
}

async function toggleUserActive(user: User) {
  try {
    await api.call('users.update', {
      id: user.id,
      active: !user.active,
    })
    await loadUsers()
  } catch (e) {
    alert(e instanceof Error ? e.message : 'Failed to update user')
  }
}
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-bold text-gray-900">Admin Panel</h1>

    <!-- Tabs -->
    <div class="border-b border-gray-200">
      <nav class="flex space-x-8">
        <button
          @click="activeTab = 'users'"
          :class="[
            'pb-4 px-1 border-b-2 font-medium text-sm',
            activeTab === 'users'
              ? 'border-blue-500 text-blue-600'
              : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
          ]"
        >
          User Management
        </button>
        <button
          @click="activeTab = 'system'"
          :class="[
            'pb-4 px-1 border-b-2 font-medium text-sm',
            activeTab === 'system'
              ? 'border-blue-500 text-blue-600'
              : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
          ]"
        >
          System Status
        </button>
      </nav>
    </div>

    <!-- Users Tab -->
    <div v-if="activeTab === 'users'">
      <div class="flex justify-between items-center mb-6">
        <h2 class="text-lg font-medium text-gray-900">Users</h2>
        <button @click="openCreateUser" class="btn btn-primary">
          Create User
        </button>
      </div>

      <div v-if="loadingUsers" class="text-center py-12">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-4 border-blue-500 border-t-transparent"></div>
      </div>

      <div v-else class="card">
        <table class="min-w-full divide-y divide-gray-200">
          <thead class="bg-gray-50">
            <tr>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Username</th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Email</th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Role</th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
              <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
            </tr>
          </thead>
          <tbody class="bg-white divide-y divide-gray-200">
            <tr v-for="user in users" :key="user.id">
              <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                {{ user.username }}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {{ user.email || '-' }}
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <span :class="[
                  'badge',
                  user.role === 'admin' ? 'badge-error' :
                  user.role === 'operator' ? 'badge-warning' : 'badge-info'
                ]">
                  {{ user.role }}
                </span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <span :class="['badge', user.active ? 'badge-success' : 'badge-gray']">
                  {{ user.active ? 'Active' : 'Inactive' }}
                </span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2">
                <button @click="openEditUser(user)" class="text-blue-600 hover:text-blue-900">
                  Edit
                </button>
                <button @click="toggleUserActive(user)" class="text-yellow-600 hover:text-yellow-900">
                  {{ user.active ? 'Disable' : 'Enable' }}
                </button>
                <button @click="deleteUser(user)" class="text-red-600 hover:text-red-900">
                  Delete
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- System Tab -->
    <div v-if="activeTab === 'system'">
      <div v-if="loadingSystem" class="text-center py-12">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-4 border-blue-500 border-t-transparent"></div>
      </div>

      <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div class="card">
          <div class="card-header">
            <h3 class="text-lg font-medium text-gray-900">Version Info</h3>
          </div>
          <div class="card-body">
            <dl class="space-y-2">
              <div>
                <dt class="text-sm text-gray-500">Name</dt>
                <dd class="font-medium text-gray-900">{{ systemVersion?.name }}</dd>
              </div>
              <div>
                <dt class="text-sm text-gray-500">Version</dt>
                <dd class="font-medium text-gray-900">{{ systemVersion?.version }}</dd>
              </div>
            </dl>
          </div>
        </div>

        <div class="card">
          <div class="card-header">
            <h3 class="text-lg font-medium text-gray-900">Health Status</h3>
          </div>
          <div class="card-body">
            <dl class="space-y-2">
              <div v-for="(value, key) in systemHealth" :key="key">
                <dt class="text-sm text-gray-500">{{ key }}</dt>
                <dd class="font-medium text-gray-900">
                  <span v-if="typeof value === 'boolean'" :class="['badge', value ? 'badge-success' : 'badge-error']">
                    {{ value ? 'Yes' : 'No' }}
                  </span>
                  <span v-else>{{ value }}</span>
                </dd>
              </div>
            </dl>
          </div>
        </div>
      </div>
    </div>

    <!-- User Form Modal -->
    <div v-if="showUserForm" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-lg shadow-xl max-w-md w-full mx-4">
        <div class="px-6 py-4 border-b border-gray-200">
          <h3 class="text-lg font-medium text-gray-900">
            {{ editingUser ? 'Edit User' : 'Create User' }}
          </h3>
        </div>
        <form @submit.prevent="saveUser" class="px-6 py-4 space-y-4">
          <div v-if="userFormError" class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
            {{ userFormError }}
          </div>

          <div>
            <label for="username" class="label">Username</label>
            <input
              id="username"
              v-model="userForm.username"
              type="text"
              class="input"
              required
              :disabled="!!editingUser"
            />
          </div>

          <div v-if="!editingUser">
            <label for="password" class="label">Password</label>
            <input
              id="password"
              v-model="userForm.password"
              type="password"
              class="input"
              required
            />
          </div>

          <div>
            <label for="email" class="label">Email</label>
            <input
              id="email"
              v-model="userForm.email"
              type="email"
              class="input"
            />
          </div>

          <div>
            <label for="role" class="label">Role</label>
            <select id="role" v-model="userForm.role" class="input">
              <option value="admin">Admin</option>
              <option value="operator">Operator</option>
              <option value="viewer">Viewer</option>
            </select>
          </div>

          <div class="flex justify-end space-x-4 pt-4">
            <button type="button" @click="showUserForm = false" class="btn btn-secondary">
              Cancel
            </button>
            <button type="submit" class="btn btn-primary" :disabled="savingUser">
              {{ savingUser ? 'Saving...' : 'Save' }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>
