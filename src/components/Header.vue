<script setup lang="ts">
import { Activity } from "lucide-vue-next";

defineProps<{
  isRunning: boolean;
}>();

defineEmits<{
  (e: "toggle"): void;
}>();
</script>

<template>
  <header
    class="h-16 flex items-center justify-between px-6 bg-[#2d2d2d]/50 border-b border-black/20 backdrop-blur-xl shrink-0"
  >
    <div class="flex items-center gap-3">
      <Activity class="w-6 h-6 text-blue-400" />
      <div>
        <h1 class="text-sm font-bold text-white tracking-tight leading-none">
          Ping Monitor
        </h1>
        <p class="text-[10px] text-slate-500 mt-1 font-medium">
          {{ isRunning ? "监控中" : "已停止" }}
        </p>
      </div>
    </div>
    <button
      @click="$emit('toggle')"
      :class="[
        'px-4 py-1.5 rounded-md text-xs font-bold transition-all flex items-center gap-2 shadow-lg',
        isRunning
          ? 'bg-[#ff4d4d] hover:bg-red-600 text-white'
          : 'bg-[#4cd964] hover:bg-green-600 text-white',
      ]"
    >
      <div class="w-2.5 h-2.5 bg-white/90 rounded-sm"></div>
      {{ isRunning ? "停止" : "开始" }}
    </button>
  </header>
</template>
