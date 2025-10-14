<template>
  <div class="about">
    <h1>What are Liquidity Pools and how do They Work?</h1>
    <!-- Tab buttons -->
    <div class="tabs flex space-x-4 mt-6">
      <button
        v-for="(tab, idx) in tabList"
        :key="tab"
        @click="activeTab = idx"
        :class="[
          'px-4 py-2 rounded transition',
          activeTab === idx
            ? 'bg-accent text-bg'
            : 'bg-transparent text-accent hover:bg-accent/20'
        ]"
      >
        {{ tab }}
      </button>
    </div>

    <!-- Tab content -->
    <div class="tab-content pt-4">
      <div v-if="activeTab === 0">
        <p>Content for **Beginner** goes here.</p>
      </div>
      <div v-else-if="activeTab === 1">
        <p>Content for **Intermediate** goes here.</p>
      </div>
      <div v-else-if="activeTab === 2">
        <p>Content for **Advanced** goes here.</p>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref } from 'vue'

const tabList = ['Beginner', 'Intermediate', 'Advanced']
const activeTab = ref(0)
</script>

<style scoped>
.bg-accent {
  background-color: var(--color-accent);
}
.text-accent {
  color: var(--color-accent);
}
.text-bg {
  color: var(--color-bg);
}
</style>
