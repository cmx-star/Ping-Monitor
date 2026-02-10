<script setup lang="ts">
import { ref, computed } from "vue";
import { Trash2, Download } from "lucide-vue-next";

interface LogEntry {
  id: string;
  timestamp: Date;
  level: "DEBUG" | "INFO" | "WARN" | "ERROR";
  message: string;
  host?: string;
}

const props = defineProps<{
  logs: LogEntry[];
}>();

defineEmits<{
  (e: "clear-logs"): void;
  (e: "export-logs"): void;
}>();

const logFilter = ref<"ALL" | "DEBUG" | "INFO" | "WARN" | "ERROR">("ALL");

const filteredLogs = computed(() => {
  if (logFilter.value === "ALL") return props.logs;
  return props.logs.filter((log) => log.level === logFilter.value);
});
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div class="flex bg-[#2d2d2d]/30 p-1 rounded-lg border border-white/5">
        <button
          v-for="level in ['ALL', 'INFO', 'WARN', 'ERROR', 'DEBUG']"
          :key="level"
          @click="logFilter = level as any"
          :class="[
            'px-3 py-1 rounded-md text-[10px] font-bold transition-all',
            logFilter === level
              ? 'bg-blue-600 text-white shadow-lg'
              : 'text-slate-500 hover:text-slate-300',
          ]"
        >
          {{ level }}
        </button>
      </div>

      <div class="flex gap-2">
        <button
          @click="$emit('clear-logs')"
          class="p-2 text-slate-400 hover:text-white hover:bg-white/5 rounded-md transition-all"
        >
          <Trash2 class="w-4 h-4" />
        </button>
        <button
          @click="$emit('export-logs')"
          class="p-2 text-slate-400 hover:text-white hover:bg-white/5 rounded-md transition-all"
        >
          <Download class="w-4 h-4" />
        </button>
      </div>
    </div>

    <div
      class="bg-[#1a1a1a] border border-white/5 rounded-xl p-4 font-mono text-[11px] text-slate-300 space-y-1 h-[500px] overflow-y-auto"
    >
      <div
        v-if="filteredLogs.length === 0"
        class="text-slate-600 italic p-4 text-center"
      >
        暂无日志
      </div>
      <div
        v-for="log in filteredLogs"
        :key="log.id"
        class="flex gap-2 border-b border-white/5 pb-1 mb-1"
      >
        <span class="text-slate-600 shrink-0"
          >[{{ new Date(log.timestamp).toLocaleTimeString() }}]</span
        >
        <span
          :class="[
            'font-bold w-12 shrink-0',
            log.level === 'INFO'
              ? 'text-blue-500'
              : log.level === 'WARN'
              ? 'text-amber-500'
              : log.level === 'ERROR'
              ? 'text-red-500'
              : 'text-slate-500',
          ]"
          >{{ log.level }}</span
        >
        <span class="break-all"
          >{{ log.host ? `[${log.host}] ` : "" }}{{ log.message }}</span
        >
      </div>
    </div>
  </div>
</template>
